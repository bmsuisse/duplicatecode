mod bench;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use duplicatecode_engine::index::Weights;
use duplicatecode_engine::{diff, find_matches, Corpus, load_units, load_units_with, units_from_file, MatchOptions};
use std::io::Read;
use std::path::PathBuf;

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
        /// Repository root (the corpus, and where the diff's paths resolve).
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        /// Diff file, or `-` for stdin.
        #[arg(long, default_value = "-")]
        diff: String,
        #[arg(long, default_value_t = 0.4)]
        threshold: f64,
        #[arg(long, default_value_t = 20)]
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
    /// Find similar unit pairs inside one repository (self-comparison).
    Scan {
        /// One or more folders (e.g. every package of a monorepo).
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// Glob to skip (gitignore syntax, repeatable), e.g. `--exclude '**/generated/**'`.
        /// `.gitignore` is honoured automatically.
        #[arg(long)]
        exclude: Vec<String>,
        #[arg(long, default_value_t = 0.6)]
        threshold: f64,
        #[arg(long, default_value_t = 20)]
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
        #[arg(long, default_value_t = 20)]
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
        Cmd::Diff { repo, diff, threshold, min_tokens, min_name, profile, min_lines, json } => {
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
            let corpus = Corpus::new(load_units(&repo));
            let opts = MatchOptions { threshold, min_tokens, min_name, min_lines, weights: profile.weights(), ..Default::default() };
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
        Cmd::Scan { paths, exclude, threshold, min_tokens, min_name, profile, min_lines, skip_tests, cross_file, pairs_out, fail_on_found, json } => {
            let mut units = Vec::new();
            for p in &paths {
                for mut u in load_units_with(p, &exclude) {
                    if paths.len() > 1 {
                        u.file = format!("{}/{}", p.display(), u.file);
                    }
                    units.push(u);
                }
            }
            let corpus = Corpus::new(units.clone());
            let opts = MatchOptions { threshold, min_tokens, top_n: 3, min_name, min_lines, skip_tests, weights: profile.weights(), ..Default::default() };
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
