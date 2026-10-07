//! Static similarity matrix for the codebase retrieval benchmark (`eval/embed_codebases.py`).
//!
//!     static-rank --ext py corpus.jsonl queries.jsonl > scores.json
//!
//! Both inputs are JSON lines with a `text` field (one function each). Output: a JSON list with one row
//! per query holding the engine's `combined` score (the `reimpl` weights, the same ones `scan --profile
//! reimpl` uses) against every corpus entry; `null` where a snippet has no parsable unit.

use duplicatecode_engine::index::{score_lean, Weights};
use duplicatecode_engine::{extract_units, Lang, Unit};
use std::path::Path;

/// Remove the indentation common to all non-empty lines (methods are cut out of their class).
fn dedent(text: &str) -> String {
    let indent = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    text.lines()
        .map(|l| if l.len() >= indent { &l[indent..] } else { l.trim_start() })
        .collect::<Vec<_>>()
        .join("\n")
}

fn load(path: &str, lang: Lang, file: &str) -> Vec<Option<Unit>> {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .lines()
        .map(|l| {
            let v: serde_json::Value = serde_json::from_str(l).expect("json line");
            let text = dedent(v["text"].as_str().or(v["code"].as_str()).unwrap_or(""));
            // the snippet is one function: take its largest unit (a method's class wrapper is not there)
            extract_units(file, lang, &text)
                .into_iter()
                .max_by_key(|u| u.token_count())
        })
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ext = args
        .iter()
        .position(|a| a == "--ext")
        .and_then(|i| args.get(i + 1))
        .expect("--ext <py|js|ts|tsx|sql>");
    let paths: Vec<&String> = args.iter().skip(1).filter(|a| a.ends_with(".jsonl")).collect();
    assert_eq!(paths.len(), 2, "usage: static-rank --ext EXT corpus.jsonl queries.jsonl");
    let file = format!("x.{ext}");
    let lang = Lang::from_path(Path::new(&file)).expect("unsupported extension");
    let corpus = load(paths[0], lang, &file);
    let queries = load(paths[1], lang, &file);
    let weights = Weights::default();
    let rows: Vec<Vec<Option<f64>>> = queries
        .iter()
        .map(|q| {
            corpus
                .iter()
                .map(|c| match (q, c) {
                    (Some(q), Some(c)) => Some(score_lean(q, c, &weights).combined),
                    _ => None,
                })
                .collect()
        })
        .collect();
    println!("{}", serde_json::to_string(&rows).unwrap());
}
