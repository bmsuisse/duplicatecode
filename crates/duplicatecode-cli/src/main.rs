mod bench;
mod groups;
mod review;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use duplicatecode_engine::index::Weights;
use duplicatecode_engine::{
    diff, find_matches, load_units, load_units_with, units_from_file, Corpus, MatchOptions,
};
use std::io::Read;
use std::path::PathBuf;

/// Optional semantic name similarity (Azure AI Foundry / Azure OpenAI / OpenAI-compatible embeddings).
/// Configure with OPENAI_API_KEY [+ OPENAI_BASE_URL, OPENAI_EMBEDDING_MODEL], COHERE_API_KEY [+ COHERE_EMBEDDING_MODEL], or AZURE_AI_FOUNDRY_ENDPOINT, AZURE_AI_FOUNDRY_API_KEY (or `az login`) and
/// AZURE_AI_FOUNDRY_EMBEDDING_DEPLOYMENT. Vectors are cached in a flat file.
#[derive(clap::Args, Clone)]
struct EmbedArgs {
    /// Embed unit names and use the cosine as an extra name-similarity signal.
    #[arg(long)]
    embeddings: bool,
    /// Embed the full text of every unit (function, class, file, statement) and blend the cosine of
    /// the two vectors into the score. Finds re-implementations that share no tokens.
    #[arg(long)]
    embed_code: bool,
    /// Weight of the code-embedding cosine in the blended score (others are scaled by 1 - weight).
    #[arg(long, default_value_t = 0.35)]
    embed_weight: f64,
    /// Characters of each unit sent to the embedding model.
    #[arg(long, default_value_t = 3000)]
    embed_max_chars: usize,
    /// Vector size requested from the service (text-embedding-3 models accept shorter vectors).
    #[arg(long, default_value_t = 256)]
    embed_dims: u32,
    /// Flat cache file (default ~/.cache/duplicatecode/embeddings.bin).
    #[arg(long)]
    embed_cache: Option<PathBuf>,
    /// Cosine at/below which two names count as unrelated. Calibrate with `embed-test`.
    #[arg(long, default_value_t = 0.5)]
    embed_floor: f64,
}

impl EmbedArgs {
    fn apply(&self, units: &mut [duplicatecode_engine::Unit]) -> Result<()> {
        if !self.embeddings && !self.embed_code {
            return Ok(());
        }
        let cfg = duplicatecode_engine::embed::EmbedConfig::from_env(Some(self.embed_dims))
            .context("--embeddings/--embed-code need OPENAI_API_KEY (optionally OPENAI_BASE_URL for compatible servers), COHERE_API_KEY or AZURE_AI_FOUNDRY_ENDPOINT / AZURE_OPENAI_ENDPOINT / DUPLICATECODE_EMBED_ENDPOINT")?;
        let path = self
            .embed_cache
            .clone()
            .unwrap_or_else(duplicatecode_engine::embed::EmbeddingCache::default_path);
        let mut cache = duplicatecode_engine::embed::EmbeddingCache::load(&path);
        let report = |what: &str, st: duplicatecode_engine::embed::EmbedStats| {
            eprintln!(
                "{what}: {} distinct ({} cached, {} fetched) via {} [{}]",
                st.distinct_names,
                st.from_cache,
                st.fetched,
                cfg.model_id(),
                path.display()
            );
        };
        if self.embeddings {
            report(
                "name embeddings",
                duplicatecode_engine::embed::embed_unit_names(units, &cfg, &mut cache)
                    .map_err(anyhow::Error::msg)?,
            );
        }
        if self.embed_code {
            report(
                "code embeddings",
                duplicatecode_engine::embed::embed_unit_code(
                    units,
                    &cfg,
                    &mut cache,
                    self.embed_max_chars,
                )
                .map_err(anyhow::Error::msg)?,
            );
        }
        Ok(())
    }

    fn weights(&self, w: Weights) -> Weights {
        let w = Weights {
            name_floor: self.embed_floor,
            ..w
        };
        if self.embed_code {
            w.with_embed(self.embed_weight)
        } else {
            w
        }
    }
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Profile {
    Copies,
    Reimpl,
}

/// Score from which a pair is reported when `--threshold` is not given. `copies` keeps its tuned
/// value; `reimpl` scores live lower (measured: ~0.25 at 1% and ~0.34 at 0.1% false-positive rate
/// on unrelated code), so it gets values matching those rates instead of the copies cut-off.
#[derive(Clone, Copy)]
enum Command {
    Diff,
    Review,
    Scan,
}

impl Profile {
    fn default_threshold(self, command: Command) -> f64 {
        match (self, command) {
            (Profile::Copies, Command::Diff) => 0.4,
            (Profile::Copies, Command::Review) => 0.45,
            (Profile::Copies, Command::Scan) => 0.6,
            (Profile::Reimpl, Command::Diff) => 0.28,
            (Profile::Reimpl, Command::Review) => 0.3,
            (Profile::Reimpl, Command::Scan) => 0.35,
        }
    }

    fn weights(self) -> Weights {
        match self {
            Profile::Copies => Weights::copies(),
            Profile::Reimpl => Weights::default(),
        }
    }
}

#[derive(Parser)]
#[command(
    version,
    about = "Detect duplicate/similar code in Python, TypeScript and C#"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List extracted units (functions, methods, classes) below a path.
    Units { path: PathBuf },
    /// Check the added code in a unified diff against the existing code in a repo.
    Diff {
        #[command(flatten)]
        embed: EmbedArgs,
        /// Repository root (the corpus, and where the diff's paths resolve).
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        /// Diff file, or `-` for stdin.
        #[arg(long, default_value = "-")]
        diff: String,
        #[arg(long)]
        threshold: Option<f64>,
        #[arg(long, default_value_t = 8)]
        min_tokens: usize,
        /// Minimum name similarity (0 = also report look-alikes with unrelated names).
        #[arg(long, default_value_t = 0.3)]
        min_name: f64,
        /// `copies`: tuned on real repos, favours same-name copy-paste. `reimpl`: structure-heavy,
        /// finds renamed re-implementations (experimental, use with --min-name 0).
        #[arg(long, value_enum, default_value_t = Profile::Copies)]
        profile: Profile,
        /// Ignore units shorter than this many lines.
        #[arg(long, default_value_t = 6)]
        min_lines: u32,
        #[arg(long)]
        json: bool,
    },
    /// Duplicate candidates as compact markdown for an LLM (or human) to verify: ranked groups,
    /// code preview, what differs, hints. The recommended entry point for AI agents.
    Review {
        #[command(flatten)]
        embed: EmbedArgs,
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[arg(long)]
        exclude: Vec<String>,
        /// Lower = more candidates (recall), higher = fewer (precision).
        #[arg(long)]
        threshold: Option<f64>,
        #[arg(long, default_value_t = 0.3)]
        min_name: f64,
        #[arg(long, default_value_t = 6)]
        min_lines: u32,
        #[arg(long)]
        skip_tests: bool,
        #[arg(long, default_value_t = 40)]
        max_groups: usize,
        /// Skip the first N groups (paging).
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// One line per group (scan hundreds cheaply, then open the interesting ones).
        #[arg(long)]
        brief: bool,
        /// Only one tier: identical | near | similar.
        #[arg(long)]
        tier: Option<String>,
        #[arg(long, default_value_t = 8)]
        preview_lines: usize,
        #[arg(long, value_enum, default_value_t = Profile::Copies)]
        profile: Profile,
    },
    /// Embed identifiers and print their cosine similarity (to check credentials and calibrate `--embed-floor`).
    EmbedTest {
        #[command(flatten)]
        embed: EmbedArgs,
        /// Two or more identifiers, e.g. `pickName pickLocalizedLabel getInitials`.
        #[arg(required = true, num_args = 2..)]
        names: Vec<String>,
    },
    /// Search existing code by description: "does something like this already exist?". Embeds the
    /// query and every unit below the paths and lists the closest units.
    Find {
        /// What the code should do, in plain language.
        query: String,
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[command(flatten)]
        embed: EmbedArgs,
        /// Number of results.
        #[arg(long, default_value_t = 5)]
        top: usize,
        /// Glob to skip (gitignore syntax, repeatable).
        #[arg(long)]
        exclude: Vec<String>,
        /// Ignore units with fewer tokens than this.
        #[arg(long, default_value_t = 8)]
        min_tokens: usize,
        #[arg(long)]
        json: bool,
    },
    /// Find copied blocks: runs of identical (normalized) statements shared by two different
    /// functions, even when the rest of the functions differ.
    Fragments {
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[arg(long)]
        exclude: Vec<String>,
        /// Minimum number of consecutive identical statements.
        #[arg(long, default_value_t = 4)]
        min_stmts: usize,
        /// Minimum normalized tokens in the shared run.
        #[arg(long, default_value_t = 30)]
        min_tokens: usize,
        /// Ignore test code.
        #[arg(long)]
        skip_tests: bool,
        /// Only report fragments between different files.
        #[arg(long)]
        cross_file: bool,
        #[arg(long)]
        json: bool,
    },
    /// Print source lines of a unit: `show path/to/file.py:10-40`.
    Show { spec: String },
    /// Find similar unit pairs inside one repository (self-comparison).
    Scan {
        #[command(flatten)]
        embed: EmbedArgs,
        /// One or more folders (e.g. every package of a monorepo).
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// Glob to skip (gitignore syntax, repeatable), e.g. `--exclude '**/generated/**'`.
        /// `.gitignore` is honoured automatically.
        #[arg(long)]
        exclude: Vec<String>,
        #[arg(long)]
        threshold: Option<f64>,
        #[arg(long, default_value_t = 8)]
        min_tokens: usize,
        /// Minimum name similarity (0 = also report look-alikes with unrelated names).
        #[arg(long, default_value_t = 0.3)]
        min_name: f64,
        /// `copies`: tuned on real repos, favours same-name copy-paste. `reimpl`: structure-heavy,
        /// finds renamed re-implementations (experimental, use with --min-name 0).
        #[arg(long, value_enum, default_value_t = Profile::Copies)]
        profile: Profile,
        /// Ignore units shorter than this many lines.
        #[arg(long, default_value_t = 6)]
        min_lines: u32,
        /// Ignore test code (about half of the noise in the judged repos, but also some real copies).
        #[arg(long)]
        skip_tests: bool,
        /// Only report pairs whose units are in different files.
        #[arg(long)]
        cross_file: bool,
        /// List individual pairs instead of grouping them into duplicate groups.
        #[arg(long = "pairs")]
        pairs_out: bool,
        /// With --pairs: say what differs (literals, calls, lines) so the shared part can be extracted.
        #[arg(long)]
        explain: bool,
        /// Exit with status 1 when anything is found (for CI).
        #[arg(long)]
        fail_on_found: bool,
        #[arg(long)]
        json: bool,
    },
    /// Evaluate detection of LLM re-implementations of real repository functions.
    ReimplEval {
        /// JSON list of {id, repo_root, file, start_line, end_line, lang}.
        #[arg(long)]
        cases: PathBuf,
        /// Directory with one sub-folder per model, each holding `<id>.py|.ts` files.
        #[arg(long)]
        impls: PathBuf,
    },
    /// Evaluate on labeled clone groups: `<root>/<dataset>/<group>/<file>`; files in one group are
    /// clones of each other (e.g. accepted CodeNet submissions of one problem). See `eval/`.
    EvalGroups {
        #[arg(long, default_value = "eval/data/codenet")]
        root: PathBuf,
        /// Print only the final `score=` line.
        #[arg(long)]
        quiet: bool,
        /// Write every pair's feature vector (TSV) for offline analysis.
        #[arg(long)]
        dump: Option<PathBuf>,
        #[command(flatten)]
        embed: EmbedArgs,
    },
    /// Build labeled React groups (component + mutated variants) for `eval-groups` from real TSX.
    MakeMutationGroups {
        #[arg(long)]
        src: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 80)]
        n: usize,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// Skip this many shuffled files first (use a different value for a disjoint holdout).
        #[arg(long, default_value_t = 0)]
        skip: usize,
    },
    /// Evaluate the detector on the LLM-implementation benchmark dataset.
    Bench {
        #[arg(long, default_value = "dataset")]
        dataset: PathBuf,
        #[arg(long, default_value_t = 8)]
        min_tokens: usize,
        /// Unrelated code (e.g. another repo) to measure false alarms against.
        #[arg(long)]
        negatives: Option<PathBuf>,
        /// Compare whole files instead of individual functions/classes.
        #[arg(long)]
        file_level: bool,
        /// Keep constructors/dunder methods in the benchmark (they are filtered by default).
        #[arg(long)]
        keep_boilerplate: bool,
        /// Also measure robustness against mechanical source mutations.
        #[arg(long)]
        mutations: bool,
    },
}

#[derive(serde::Serialize)]
struct Group {
    score: f64,
    units: Vec<duplicatecode_engine::index::UnitRef>,
}

/// Connected components of the similar-pair graph, best group first.
fn group_pairs(pairs: &[duplicatecode_engine::Match]) -> Vec<Group> {
    use std::collections::HashMap;
    let mut idx: HashMap<String, usize> = HashMap::new();
    let mut refs: Vec<duplicatecode_engine::index::UnitRef> = Vec::new();
    let mut parent: Vec<usize> = Vec::new();
    let mut id =
        |u: &duplicatecode_engine::index::UnitRef, refs: &mut Vec<_>, parent: &mut Vec<usize>| {
            *idx.entry(format!("{}:{}", u.file, u.start_line))
                .or_insert_with(|| {
                    refs.push(u.clone());
                    parent.push(parent.len());
                    parent.len() - 1
                })
        };
    fn find(p: &mut Vec<usize>, x: usize) -> usize {
        if p[x] != x {
            let r = find(p, p[x]);
            p[x] = r;
        }
        p[x]
    }
    let mut edges = Vec::new();
    for m in pairs {
        let a = id(&m.query, &mut refs, &mut parent);
        let b = id(&m.candidate, &mut refs, &mut parent);
        edges.push((a, b, m.scores.combined));
        let (ra, rb) = (find(&mut parent, a), find(&mut parent, b));
        parent[ra] = rb;
    }
    let mut best: HashMap<usize, f64> = HashMap::new();
    for (a, _, s) in &edges {
        let r = find(&mut parent, *a);
        let e = best.entry(r).or_insert(0.0);
        *e = e.max(*s);
    }
    let mut members: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..refs.len() {
        let r = find(&mut parent, i);
        members.entry(r).or_default().push(i);
    }
    let mut groups: Vec<Group> = members
        .into_iter()
        .map(|(r, ms)| {
            let mut units: Vec<_> = ms.into_iter().map(|i| refs[i].clone()).collect();
            units.sort_by(|a, b| (&a.file, a.start_line).cmp(&(&b.file, b.start_line)));
            Group {
                score: best[&r],
                units,
            }
        })
        .collect();
    groups.sort_by(|a, b| b.score.total_cmp(&a.score));
    groups
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Units { path } => {
            for u in load_units(&path) {
                println!(
                    "{}:{}-{} {} {} ({} tokens)",
                    u.file,
                    u.start_line,
                    u.end_line,
                    u.kind,
                    u.name,
                    u.token_count()
                );
            }
        }
        Cmd::Diff {
            embed,
            repo,
            diff,
            threshold,
            min_tokens,
            min_name,
            profile,
            min_lines,
            json,
        } => {
            let threshold = threshold.unwrap_or(profile.default_threshold(Command::Diff));
            let text = if diff == "-" {
                let mut s = String::new();
                std::io::stdin().read_to_string(&mut s)?;
                s
            } else {
                std::fs::read_to_string(&diff).with_context(|| format!("reading {diff}"))?
            };
            let mut queries = Vec::new();
            for (file, ranges) in diff::added_ranges(&text) {
                let units = units_from_file(&repo, &repo.join(&file));
                queries.extend(units.into_iter().filter(|u| {
                    ranges
                        .iter()
                        .any(|&(s, e)| s <= u.end_line && u.start_line <= e)
                }));
            }
            let mut corpus_units = load_units(&repo);
            embed.apply(&mut corpus_units)?;
            embed.apply(&mut queries)?;
            let corpus = Corpus::new(corpus_units);
            let opts = MatchOptions {
                threshold,
                min_tokens,
                min_name,
                min_lines,
                weights: embed.weights(profile.weights()),
                ..Default::default()
            };
            // when both sides are new code the pair shows up twice; report it once
            let mut seen = std::collections::HashSet::new();
            let matches: Vec<_> = find_matches(&queries, &corpus, opts)
                .into_iter()
                .filter(|m| {
                    let a = format!("{}:{}", m.query.file, m.query.start_line);
                    let b = format!("{}:{}", m.candidate.file, m.candidate.start_line);
                    seen.insert(if a < b { (a, b) } else { (b, a) })
                })
                .collect();
            if json {
                println!("{}", serde_json::to_string_pretty(&matches)?);
            } else if matches.is_empty() {
                println!(
                    "no similar code found ({} new/changed units checked)",
                    queries.len()
                );
            } else {
                for m in &matches {
                    println!(
                        "{}:{} {}  ~  {}:{} {}  combined={:.2} (structure={:.2} name={:.2} callees={:.2})",
                        m.query.file, m.query.start_line, m.query.name,
                        m.candidate.file, m.candidate.start_line, m.candidate.name,
                        m.scores.combined, m.scores.structural, m.scores.name, m.scores.callees,
                    );
                }
            }
        }
        Cmd::Review {
            embed,
            paths,
            exclude,
            threshold,
            min_name,
            min_lines,
            skip_tests,
            max_groups,
            offset,
            brief,
            tier,
            preview_lines,
            profile,
        } => {
            let threshold = threshold.unwrap_or(profile.default_threshold(Command::Review));
            let mut units = Vec::new();
            let mut src = std::collections::HashMap::new();
            for p in &paths {
                for mut u in load_units_with(p, &exclude) {
                    let real = p.join(&u.file);
                    if paths.len() > 1 {
                        u.file = format!("{}/{}", p.display(), u.file);
                    }
                    src.insert(u.file.clone(), real);
                    units.push(u);
                }
            }
            embed.apply(&mut units)?;
            let o = review::ReviewOptions {
                threshold,
                min_name,
                min_lines,
                skip_tests,
                max_groups,
                offset,
                brief,
                tier,
                preview_lines,
                weights: embed.weights(profile.weights()),
            };
            print!("{}", review::run(units, &src, &o));
        }
        Cmd::MakeMutationGroups {
            src,
            out,
            n,
            seed,
            skip,
        } => groups::make_mutation_groups(&src, &out, n, seed, skip)?,
        Cmd::EvalGroups {
            root,
            quiet,
            dump,
            embed,
        } => groups::run(
            &root,
            quiet,
            dump.as_deref(),
            &|u| embed.apply(u),
            embed.weights(Weights::default()),
        )?,
        Cmd::EmbedTest { embed, names } => {
            let cfg = duplicatecode_engine::embed::EmbedConfig::from_env(Some(embed.embed_dims))
                .context(
                    "set OPENAI_API_KEY (+ OPENAI_BASE_URL), or AZURE_AI_FOUNDRY_ENDPOINT (+ AZURE_AI_FOUNDRY_API_KEY or `az login`)",
                )?;
            let path = embed
                .embed_cache
                .clone()
                .unwrap_or_else(duplicatecode_engine::embed::EmbeddingCache::default_path);
            let mut cache = duplicatecode_engine::embed::EmbeddingCache::load(&path);
            let mut units: Vec<_> = names
                .iter()
                .filter_map(|n| {
                    duplicatecode_engine::extract_units(
                        "x.py",
                        duplicatecode_engine::Lang::Python,
                        &format!("def {n}():\n    return 1\n"),
                    )
                    .into_iter()
                    .next()
                })
                .collect();
            let st = duplicatecode_engine::embed::embed_unit_names(&mut units, &cfg, &mut cache)
                .map_err(anyhow::Error::msg)?;
            println!(
                "model {} ({} fetched, {} cached)",
                cfg.model_id(),
                st.fetched,
                st.from_cache
            );
            for i in 0..units.len() {
                for j in i + 1..units.len() {
                    let (a, b) = (
                        units[i].name_vec.as_ref().unwrap(),
                        units[j].name_vec.as_ref().unwrap(),
                    );
                    let cos: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                    println!("{:.3}  {}  ~  {}", cos, units[i].name, units[j].name);
                }
            }
        }
        Cmd::Show { spec } => {
            let (file, range) = spec.rsplit_once(':').context("expected path:START-END")?;
            let (a, b) = range.split_once('-').context("expected START-END")?;
            let (a, b): (usize, usize) = (a.parse()?, b.parse()?);
            let text = std::fs::read_to_string(file).with_context(|| format!("reading {file}"))?;
            for (i, l) in text
                .lines()
                .enumerate()
                .skip(a.saturating_sub(1))
                .take(b + 1 - a)
            {
                println!("{:>5}| {l}", i + 1);
            }
        }
        Cmd::Fragments {
            paths,
            exclude,
            min_stmts,
            min_tokens,
            skip_tests,
            cross_file,
            json,
        } => {
            let mut units = Vec::new();
            for p in &paths {
                units.extend(load_units_with(p, &exclude));
            }
            units.retain(|u| !u.boilerplate && !(skip_tests && u.is_test));
            let mut found = duplicatecode_engine::fragments::find_fragments(
                &units,
                duplicatecode_engine::fragments::FragmentOptions {
                    min_stmts,
                    min_tokens,
                    ..Default::default()
                },
            );
            if cross_file {
                found.retain(|f| f.a.file != f.b.file);
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&found)?);
            } else {
                for f in &found {
                    println!(
                        "{} stmts, {} tokens  {}:{}-{} ({}, {:.0}%)  <->  {}:{}-{} ({}, {:.0}%)",
                        f.statements,
                        f.tokens,
                        f.a.file,
                        f.a.start_line,
                        f.a.end_line,
                        f.a.unit,
                        f.a.coverage * 100.0,
                        f.b.file,
                        f.b.start_line,
                        f.b.end_line,
                        f.b.unit,
                        f.b.coverage * 100.0
                    );
                }
                eprintln!("{} fragment pair(s)", found.len());
            }
        }
        Cmd::Find {
            query,
            paths,
            mut embed,
            top,
            exclude,
            min_tokens,
            json,
        } => {
            embed.embed_code = true; // searching by description needs unit embeddings
            let mut units = Vec::new();
            for p in &paths {
                units.extend(load_units_with(p, &exclude));
            }
            units.retain(|u| u.token_count() >= min_tokens && !u.boilerplate);
            embed.apply(&mut units)?;
            let cfg = duplicatecode_engine::embed::EmbedConfig::from_env(Some(embed.embed_dims))
                .context("no embedding provider configured")?;
            let q = duplicatecode_engine::embed::embed_query(&cfg, &query)
                .map_err(anyhow::Error::msg)?;
            let mut hits: Vec<(f64, &duplicatecode_engine::Unit)> = units
                .iter()
                .filter_map(|u| {
                    let v = u.vec.as_ref()?;
                    Some((duplicatecode_engine::similarity::dot(&q, v)?, u))
                })
                .collect();
            hits.sort_by(|a, b| b.0.total_cmp(&a.0));
            hits.truncate(top);
            if json {
                let rows: Vec<_> = hits
                    .iter()
                    .map(|(s, u)| {
                        serde_json::json!({"score": s, "file": u.file, "name": u.name,
                            "kind": u.kind, "start_line": u.start_line, "end_line": u.end_line})
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else {
                for (s, u) in hits {
                    println!(
                        "{s:.3}  {}:{}-{}  {} {}",
                        u.file, u.start_line, u.end_line, u.kind, u.name
                    );
                }
            }
        }
        Cmd::Scan {
            embed,
            paths,
            exclude,
            threshold,
            min_tokens,
            min_name,
            profile,
            min_lines,
            skip_tests,
            cross_file,
            pairs_out,
            explain,
            fail_on_found,
            json,
        } => {
            let threshold = threshold.unwrap_or(profile.default_threshold(Command::Scan));
            let mut units = Vec::new();
            for p in &paths {
                for mut u in load_units_with(p, &exclude) {
                    if paths.len() > 1 {
                        u.file = format!("{}/{}", p.display(), u.file);
                    }
                    units.push(u);
                }
            }
            embed.apply(&mut units)?;
            let corpus = Corpus::new(units.clone());
            let opts = MatchOptions {
                threshold,
                min_tokens,
                top_n: 3,
                min_name,
                min_lines,
                skip_tests,
                weights: embed.weights(profile.weights()),
                ..Default::default()
            };
            let mut seen = std::collections::HashSet::new();
            use rayon::prelude::*;
            let mut pairs: Vec<_> = units
                .par_iter()
                .flat_map_iter(|u| find_matches(std::slice::from_ref(u), &corpus, opts))
                .collect::<Vec<_>>()
                .into_iter()
                .filter(|m| !cross_file || m.query.file != m.candidate.file)
                .filter(|m| {
                    let a = format!("{}:{}", m.query.file, m.query.start_line);
                    let b = format!("{}:{}", m.candidate.file, m.candidate.start_line);
                    seen.insert(if a < b { (a, b) } else { (b, a) })
                })
                .collect();
            pairs.sort_by(|a, b| b.scores.combined.total_cmp(&a.scores.combined));
            let groups = group_pairs(&pairs);
            let by_place: std::collections::HashMap<String, &duplicatecode_engine::Unit> = units
                .iter()
                .map(|u| (format!("{}:{}", u.file, u.start_line), u))
                .collect();
            let explanation = |m: &duplicatecode_engine::Match| {
                let a = by_place.get(&format!("{}:{}", m.query.file, m.query.start_line))?;
                let b =
                    by_place.get(&format!("{}:{}", m.candidate.file, m.candidate.start_line))?;
                Some(duplicatecode_engine::explain::explain(a, b))
            };
            if pairs_out {
                if json {
                    if explain {
                        let rows: Vec<_> = pairs
                            .iter()
                            .map(|m| serde_json::json!({"match": m, "explanation": explanation(m)}))
                            .collect();
                        println!("{}", serde_json::to_string_pretty(&rows)?);
                    } else {
                        println!("{}", serde_json::to_string_pretty(&pairs)?);
                    }
                } else {
                    println!(
                        "{} units scanned, {} similar pairs (>= {threshold})",
                        units.len(),
                        pairs.len()
                    );
                    for m in &pairs {
                        println!(
                            "{:.2}  {}:{}-{} {}  ~  {}:{}-{} {}",
                            m.scores.combined,
                            m.query.file,
                            m.query.start_line,
                            m.query.end_line,
                            m.query.name,
                            m.candidate.file,
                            m.candidate.start_line,
                            m.candidate.end_line,
                            m.candidate.name
                        );
                        if let Some(e) = explain.then(|| explanation(m)).flatten() {
                            let text = e.summary();
                            if !text.is_empty() {
                                println!("      differs: {text}");
                            }
                        }
                    }
                }
            } else if json {
                println!("{}", serde_json::to_string_pretty(&groups)?);
            } else {
                println!(
                    "{} units scanned, {} duplicate groups ({} pairs, score >= {threshold})",
                    units.len(),
                    groups.len(),
                    pairs.len()
                );
                for (i, g) in groups.iter().enumerate() {
                    println!(
                        "\ngroup {} — {} units, best score {:.2}",
                        i + 1,
                        g.units.len(),
                        g.score
                    );
                    for u in &g.units {
                        println!(
                            "  {}:{}-{}  {} {}",
                            u.file, u.start_line, u.end_line, u.kind, u.name
                        );
                    }
                }
            }
            if fail_on_found && !pairs.is_empty() {
                std::process::exit(1);
            }
        }
        Cmd::ReimplEval { cases, impls } => bench::reimpl_eval(&cases, &impls)?,
        Cmd::Bench {
            dataset,
            min_tokens,
            negatives,
            file_level,
            keep_boilerplate,
            mutations,
        } => bench::run(
            &dataset,
            min_tokens,
            negatives.as_deref(),
            file_level,
            keep_boilerplate,
            mutations,
        )?,
    }
    Ok(())
}
