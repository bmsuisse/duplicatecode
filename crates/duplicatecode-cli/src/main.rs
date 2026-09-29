mod bench;
mod review;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use duplicatecode_engine::index::Weights;
use duplicatecode_engine::{diff, find_matches, Corpus, load_units, load_units_with, units_from_file, MatchOptions};
use std::io::Read;
use std::path::PathBuf;

/// Optional semantic name similarity (Azure AI Foundry / Azure OpenAI / OpenAI-compatible embeddings).
/// Configure with AZURE_AI_FOUNDRY_ENDPOINT, AZURE_AI_FOUNDRY_API_KEY (or `az login`) and
/// AZURE_AI_FOUNDRY_EMBEDDING_DEPLOYMENT. Vectors are cached in a flat file.
#[derive(clap::Args, Clone)]
struct EmbedArgs {
    /// Embed unit names and use the cosine as an extra name-similarity signal.
    #[arg(long)]
    embeddings: bool,
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
        if !self.embeddings {
            return Ok(());
        }
        let cfg = duplicatecode_engine::embed::EmbedConfig::from_env(Some(self.embed_dims))
            .context("--embeddings needs AZURE_AI_FOUNDRY_ENDPOINT (or AZURE_OPENAI_ENDPOINT / DUPLICATECODE_EMBED_ENDPOINT)")?;
        let path = self.embed_cache.clone().unwrap_or_else(duplicatecode_engine::embed::EmbeddingCache::default_path);
        let mut cache = duplicatecode_engine::embed::EmbeddingCache::load(&path);
        let st = duplicatecode_engine::embed::embed_unit_names(units, &cfg, &mut cache).map_err(anyhow::Error::msg)?;
        eprintln!(
            "embeddings: {} distinct names ({} cached, {} fetched) via {} [{}]",
            st.distinct_names, st.from_cache, st.fetched, cfg.model_id(), path.display()
        );
        Ok(())
    }

    fn weights(&self, w: Weights) -> Weights {
        Weights { name_floor: self.embed_floor, ..w }
    }
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Profile {
    Copies,
    Reimpl,
}

impl Profile {
    fn weights(self) -> Weights {
        match self {
            Profile::Copies => Weights::copies(),
            Profile::Reimpl => Weights::default(),
        }
    }
}

#[derive(Parser)]
#[command(version, about = "Detect duplicate/similar code in Python and TypeScript")]
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
        #[arg(long, default_value_t = 0.4)]
        threshold: f64,
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
        #[arg(long, default_value_t = 0.45)]
        threshold: f64,
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
        #[arg(long, default_value_t = 0.6)]
        threshold: f64,
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
    let mut id = |u: &duplicatecode_engine::index::UnitRef, refs: &mut Vec<_>, parent: &mut Vec<usize>| {
        *idx.entry(format!("{}:{}", u.file, u.start_line)).or_insert_with(|| {
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
            Group { score: best[&r], units }
        })
        .collect();
    groups.sort_by(|a, b| b.score.total_cmp(&a.score));
    groups
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Units { path } => {
            for u in load_units(&path) {
                println!("{}:{}-{} {} {} ({} tokens)", u.file, u.start_line, u.end_line, u.kind, u.name, u.token_count());
            }
        }
        Cmd::Diff { embed, repo, diff, threshold, min_tokens, min_name, profile, min_lines, json } => {
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
                    ranges.iter().any(|&(s, e)| s <= u.end_line && u.start_line <= e)
                }));
            }
            let mut corpus_units = load_units(&repo);
            embed.apply(&mut corpus_units)?;
            embed.apply(&mut queries)?;
            let corpus = Corpus::new(corpus_units);
            let opts = MatchOptions { threshold, min_tokens, min_name, min_lines, weights: embed.weights(profile.weights()), ..Default::default() };
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
                println!("no similar code found ({} new/changed units checked)", queries.len());
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
        Cmd::Review { embed, paths, exclude, threshold, min_name, min_lines, skip_tests, max_groups, offset, brief, tier, preview_lines, profile } => {
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
            let o = review::ReviewOptions { threshold, min_name, min_lines, skip_tests, max_groups, offset, brief, tier, preview_lines, weights: embed.weights(profile.weights()) };
            print!("{}", review::run(units, &src, &o));
        }
        Cmd::EmbedTest { embed, names } => {
            let cfg = duplicatecode_engine::embed::EmbedConfig::from_env(Some(embed.embed_dims))
                .context("set AZURE_AI_FOUNDRY_ENDPOINT (+ AZURE_AI_FOUNDRY_API_KEY or `az login`)")?;
            let path = embed.embed_cache.clone().unwrap_or_else(duplicatecode_engine::embed::EmbeddingCache::default_path);
            let mut cache = duplicatecode_engine::embed::EmbeddingCache::load(&path);
            let mut units: Vec<_> = names
                .iter()
                .filter_map(|n| duplicatecode_engine::extract_units("x.py", duplicatecode_engine::Lang::Python, &format!("def {n}():\n    return 1\n")).into_iter().next())
                .collect();
            let st = duplicatecode_engine::embed::embed_unit_names(&mut units, &cfg, &mut cache).map_err(anyhow::Error::msg)?;
            println!("model {} ({} fetched, {} cached)", cfg.model_id(), st.fetched, st.from_cache);
            for i in 0..units.len() {
                for j in i + 1..units.len() {
                    let (a, b) = (units[i].name_vec.as_ref().unwrap(), units[j].name_vec.as_ref().unwrap());
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
            for (i, l) in text.lines().enumerate().skip(a.saturating_sub(1)).take(b + 1 - a) {
                println!("{:>5}| {l}", i + 1);
            }
        }
        Cmd::Scan { embed, paths, exclude, threshold, min_tokens, min_name, profile, min_lines, skip_tests, cross_file, pairs_out, fail_on_found, json } => {
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
            let opts = MatchOptions { threshold, min_tokens, top_n: 3, min_name, min_lines, skip_tests, weights: embed.weights(profile.weights()), ..Default::default() };
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
            if pairs_out {
                if json {
                    println!("{}", serde_json::to_string_pretty(&pairs)?);
                } else {
                    println!("{} units scanned, {} similar pairs (>= {threshold})", units.len(), pairs.len());
                    for m in &pairs {
                        println!(
                            "{:.2}  {}:{}-{} {}  ~  {}:{}-{} {}",
                            m.scores.combined, m.query.file, m.query.start_line, m.query.end_line, m.query.name,
                            m.candidate.file, m.candidate.start_line, m.candidate.end_line, m.candidate.name
                        );
                    }
                }
            } else if json {
                println!("{}", serde_json::to_string_pretty(&groups)?);
            } else {
                println!("{} units scanned, {} duplicate groups ({} pairs, score >= {threshold})", units.len(), groups.len(), pairs.len());
                for (i, g) in groups.iter().enumerate() {
                    println!("\ngroup {} — {} units, best score {:.2}", i + 1, g.units.len(), g.score);
                    for u in &g.units {
                        println!("  {}:{}-{}  {} {}", u.file, u.start_line, u.end_line, u.kind, u.name);
                    }
                }
            }
            if fail_on_found && !pairs.is_empty() {
                std::process::exit(1);
            }
        }
        Cmd::ReimplEval { cases, impls } => bench::reimpl_eval(&cases, &impls)?,
        Cmd::Bench { dataset, min_tokens, negatives, file_level, keep_boilerplate, mutations } => {
            bench::run(&dataset, min_tokens, negatives.as_deref(), file_level, keep_boilerplate, mutations)?
        },
    }
    Ok(())
}
