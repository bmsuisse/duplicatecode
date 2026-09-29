//! Scoring and matching of query units against a corpus.

use crate::similarity::{containment, jaccard};
use crate::units::Unit;

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct Scores {
    /// Jaccard of normalized token k-grams.
    pub structural: f64,
    /// Overlap coefficient of the same k-grams (catches one unit embedded in a bigger one).
    pub containment: f64,
    /// Jaccard of identifier subwords in the unit names.
    pub name: f64,
    /// Jaccard of called-function names.
    pub callees: f64,
    pub combined: f64,
}

pub fn score(a: &Unit, b: &Unit) -> Scores {
    let structural = jaccard(&a.fingerprint, &b.fingerprint);
    let containment = containment(&a.fingerprint, &b.fingerprint);
    let name = jaccard(&a.name_parts, &b.name_parts);
    let callees = jaccard(&a.callees, &b.callees);
    let combined = 0.6 * structural + 0.2 * name + 0.2 * callees;
    Scores { structural, containment, name, callees, combined }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct UnitRef {
    pub file: String,
    pub name: String,
    pub kind: String,
    pub start_line: u32,
    pub end_line: u32,
}

impl From<&Unit> for UnitRef {
    fn from(u: &Unit) -> Self {
        UnitRef {
            file: u.file.clone(),
            name: u.name.clone(),
            kind: u.kind.clone(),
            start_line: u.start_line,
            end_line: u.end_line,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Match {
    pub query: UnitRef,
    pub candidate: UnitRef,
    pub scores: Scores,
}

#[derive(Clone, Copy, Debug)]
pub struct MatchOptions {
    /// Minimum `combined` score to report.
    pub threshold: f64,
    /// Units with fewer normalized tokens are ignored (getters, one-liners).
    pub min_tokens: usize,
    /// Max matches reported per query unit.
    pub top_n: usize,
}

impl Default for MatchOptions {
    fn default() -> Self {
        MatchOptions { threshold: 0.5, min_tokens: 20, top_n: 3 }
    }
}

fn same_place(a: &Unit, b: &Unit) -> bool {
    a.file == b.file && a.start_line <= b.end_line && b.start_line <= a.end_line
}

/// For each query unit, the best-scoring corpus units of the same language family.
pub fn find_matches(queries: &[Unit], corpus: &[Unit], opts: MatchOptions) -> Vec<Match> {
    let mut out = Vec::new();
    for q in queries.iter().filter(|q| q.token_count() >= opts.min_tokens) {
        let mut hits: Vec<Match> = corpus
            .iter()
            .filter(|c| {
                c.token_count() >= opts.min_tokens
                    && c.lang.family() == q.lang.family()
                    && !same_place(q, c)
            })
            .map(|c| Match { query: q.into(), candidate: c.into(), scores: score(q, c) })
            .filter(|m| m.scores.combined >= opts.threshold)
            .collect();
        hits.sort_by(|a, b| b.scores.combined.total_cmp(&a.scores.combined));
        hits.truncate(opts.top_n);
        out.extend(hits);
    }
    out
}

/// Extract units from a single file (language from the extension); empty if unsupported/unreadable.
pub fn units_from_file(root: &std::path::Path, path: &std::path::Path) -> Vec<Unit> {
    let Some(lang) = crate::lang::Lang::from_path(path) else {
        return Vec::new();
    };
    let Ok(source) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/");
    crate::units::extract_units(&rel, lang, &source)
}

const SKIP_DIRS: &[&str] =
    &[".git", "node_modules", "target", ".venv", "venv", "__pycache__", "dist", "build", ".codegraph"];

/// All units of all supported files below `root`.
pub fn load_units(root: &std::path::Path) -> Vec<Unit> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            e.depth() == 0
                || !e.file_name().to_str().is_some_and(|n| SKIP_DIRS.contains(&n))
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .flat_map(|e| units_from_file(root, e.path()))
        .collect()
}
