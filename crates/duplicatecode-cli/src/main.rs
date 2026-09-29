mod bench;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use duplicatecode_engine::index::Weights;
use duplicatecode_engine::{diff, find_matches, Corpus, load_units, units_from_file, MatchOptions};
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
        #[arg(long)]
        json: bool,
    },
    /// Find similar unit pairs inside one repository (self-comparison).
    Scan {
        path: PathBuf,
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
        /// Only report pairs whose units are in different files.
        #[arg(long)]
        cross_file: bool,
        #[arg(long)]
        json: bool,
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
        Cmd::Scan { path, threshold, min_tokens, min_name, profile, min_lines, cross_file, json } => {
            let units = load_units(&path);
            let corpus = Corpus::new(units.clone());
            let opts = MatchOptions { threshold, min_tokens, top_n: 1, min_name, min_lines, weights: profile.weights(), ..Default::default() };
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
            if json {
                println!("{}", serde_json::to_string_pretty(&pairs)?);
            } else {
                println!("{} units, {} similar pairs (>= {threshold})", units.len(), pairs.len());
                for m in &pairs {
                    println!(
                        "{:.2}  {}:{}-{} {}  ~  {}:{}-{} {}",
                        m.scores.combined, m.query.file, m.query.start_line, m.query.end_line, m.query.name,
                        m.candidate.file, m.candidate.start_line, m.candidate.end_line, m.candidate.name
                    );
                }
            }
        }
        Cmd::Bench { dataset, min_tokens, negatives, file_level, keep_boilerplate, mutations } => {
            bench::run(&dataset, min_tokens, negatives.as_deref(), file_level, keep_boilerplate, mutations)?
        },
    }
    Ok(())
}
