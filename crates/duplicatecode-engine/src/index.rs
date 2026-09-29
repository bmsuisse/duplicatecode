//! Scoring and matching of query units against a corpus.

use crate::similarity::{containment, cosine, jaccard, lcs_ratio, multiset_dice};
use crate::units::Unit;

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct Scores {
    /// Jaccard of normalized token 4-grams.
    pub structural: f64,
    /// Jaccard of normalized token 2-grams (tolerates reordering).
    pub loose: f64,
    /// Overlap coefficient of the 4-grams (one unit embedded in a bigger one).
    pub containment: f64,
    /// Cosine of syntax-node-kind histograms.
    pub kinds: f64,
    /// Jaccard of notable literals; 0 when either side has none.
    pub literals: f64,
    /// Jaccard of called and accessed API names.
    pub api: f64,
    /// Jaccard of identifier subwords in the unit names.
    pub name: f64,
    /// Jaccard of called-function names.
    pub callees: f64,
    /// Multiset Dice of normalized statements (order-insensitive).
    pub stmt_exact: f64,
    /// Multiset Dice of coarse statement shapes.
    pub stmt_shape: f64,
    /// Order-aware alignment (LCS) of statement shapes.
    pub stmt_lcs: f64,
    pub combined: f64,
}

impl Scores {
    /// Feature vector in the order used by [`Weights`].
    pub fn features(&self) -> [f64; N_FEATURES] {
        [
            self.structural,
            self.loose,
            self.kinds,
            self.literals,
            self.api,
            self.name,
            self.callees,
            self.stmt_exact,
            self.stmt_shape,
            self.stmt_lcs,
        ]
    }
}

pub const N_FEATURES: usize = 10;
pub const FEATURE_NAMES: [&str; N_FEATURES] =
    [
    "structural", "loose", "kinds", "literals", "api", "name", "callees", "stmt_exact", "stmt_shape",
    "stmt_lcs",
];

/// Linear scoring weights (bias first); fitted by `duplicatecode bench --fit`.
#[derive(Clone, Copy, Debug)]
pub struct Weights {
    pub bias: f64,
    pub w: [f64; N_FEATURES],
}

impl Default for Weights {
    /// Re-implementation profile: mostly structure, so renamed rewrites still match.
    fn default() -> Self {
        Weights { bias: 0.0, w: [0.6, 0.0, 0.0, 0.0, 0.0, 0.2, 0.2, 0.0, 0.0, 0.0] }
    }
}

impl Weights {
    /// Copy-paste profile, chosen on ~190 hand-judged pairs from three real repositories
    /// (AUC 0.80 -> 0.88; better on each repo when fitted on the other two). Name similarity,
    /// identical statements and shared literals separate real copies from convention-driven
    /// look-alikes (CRUD endpoints, thin wrappers) far better than raw token overlap.
    pub fn copies() -> Self {
        Weights { bias: 0.0, w: [0.09, 0.0, 0.0, 0.18, 0.09, 0.36, 0.0, 0.27, 0.0, 0.0] }
    }
}

pub fn score(a: &Unit, b: &Unit) -> Scores {
    score_with(a, b, &Weights::default())
}

pub fn score_with(a: &Unit, b: &Unit, weights: &Weights) -> Scores {
    let mut s = Scores {
        structural: jaccard(&a.fingerprint, &b.fingerprint),
        loose: jaccard(&a.fingerprint2, &b.fingerprint2),
        containment: containment(&a.fingerprint, &b.fingerprint),
        kinds: cosine(&a.kinds, &b.kinds),
        literals: jaccard(&a.literals, &b.literals),
        api: jaccard(&a.api, &b.api),
        name: jaccard(&a.name_parts, &b.name_parts),
        callees: jaccard(&a.callees, &b.callees),
        stmt_exact: multiset_dice(&a.stmts_exact, &b.stmts_exact),
        stmt_shape: multiset_dice(&a.stmts_shape, &b.stmts_shape),
        stmt_lcs: lcs_ratio(&a.shape_seq, &b.shape_seq),
        combined: 0.0,
    };
    s.combined =
        weights.bias + s.features().iter().zip(weights.w).map(|(f, w)| f * w).sum::<f64>();
    s
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
    /// Units shorter than this many lines only match when they are near-exact copies with the
    /// same name (tiny wrappers around different endpoints are otherwise the top false alarm).
    pub min_lines: u32,
    /// Minimum name similarity: in real code, look-alikes with unrelated names are almost always
    /// convention-driven boilerplate. Set to 0 to hunt for renamed re-implementations.
    pub min_name: f64,
    /// Ignore test code entirely (as query and candidate).
    pub skip_tests: bool,
    /// Scoring weights (see [`Weights::default`] and [`Weights::copies`]).
    pub weights: Weights,
    /// Ignore constructors, dunder methods and similar boilerplate (as query and as candidate).
    pub skip_boilerplate: bool,
}

impl Default for MatchOptions {
    fn default() -> Self {
        MatchOptions { threshold: 0.5, min_tokens: 8, top_n: 3, min_lines: 6, min_name: 0.3, weights: Weights::default(), skip_tests: false, skip_boilerplate: true }
    }
}

fn same_place(a: &Unit, b: &Unit) -> bool {
    a.file == b.file && a.start_line <= b.end_line && b.start_line <= a.end_line
}

/// A searchable set of units with an inverted index over their k-gram fingerprints.
pub struct Corpus {
    units: Vec<Unit>,
    postings: std::collections::HashMap<u64, Vec<u32>>,
    /// name subword -> units (used when a minimum name similarity is required)
    name_postings: std::collections::HashMap<String, Vec<u32>>,
}

impl Corpus {
    pub fn new(units: Vec<Unit>) -> Corpus {
        let mut postings: std::collections::HashMap<u64, Vec<u32>> = Default::default();
        for (i, u) in units.iter().enumerate() {
            for h in &u.fingerprint {
                postings.entry(*h).or_default().push(i as u32);
            }
        }
        let mut name_postings: std::collections::HashMap<String, Vec<u32>> = Default::default();
        for (i, u) in units.iter().enumerate() {
            for part in &u.name_parts {
                name_postings.entry(part.clone()).or_default().push(i as u32);
            }
        }
        Corpus { units, postings, name_postings }
    }

    pub fn units(&self) -> &[Unit] {
        &self.units
    }

    /// Units that could still reach `threshold`. With the default weights the score is
    /// `w0 * jaccard + (at most rest)`, so a candidate needs `jaccard >= (threshold - rest) / w0`,
    /// and `jaccard >= s` implies at least `s * |A|` shared k-grams. Exact, not heuristic.
    fn candidates(&self, q: &Unit, opts: &MatchOptions) -> Vec<u32> {
        if opts.min_name > 0.0 {
            // a name similarity > 0 needs a shared subword: far fewer candidates than k-grams
            let mut seen = vec![false; self.units.len()];
            let mut out = Vec::new();
            for part in &q.name_parts {
                for &i in self.name_postings.get(part).into_iter().flatten() {
                    if !seen[i as usize] {
                        seen[i as usize] = true;
                        out.push(i);
                    }
                }
            }
            return out;
        }
        let (threshold, w) = (opts.threshold, &opts.weights);
        let rest: f64 = w.w[1..].iter().filter(|x| **x > 0.0).sum::<f64>() + w.bias.max(0.0);
        let min_jaccard = if w.w[0] > 0.0 { ((threshold - rest) / w.w[0]).max(0.0) } else { 0.0 };
        let min_shared = ((min_jaccard * q.fingerprint.len() as f64).ceil() as u32).max(1);
        let mut counts: std::collections::HashMap<u32, u32> = Default::default();
        for h in &q.fingerprint {
            for &i in self.postings.get(h).into_iter().flatten() {
                *counts.entry(i).or_insert(0) += 1;
            }
        }
        counts.into_iter().filter(|(_, c)| *c >= min_shared).map(|(i, _)| i).collect()
    }

    /// Best-scoring units for `q`, highest `combined` first, restricted to the same language
    /// family and excluding `q`'s own location.
    pub fn best_for(&self, q: &Unit, opts: MatchOptions) -> Vec<(&Unit, Scores)> {
        let mut hits: Vec<(&Unit, Scores)> = self
            .candidates(q, &opts)
            .into_iter()
            .map(|i| &self.units[i as usize])
            .filter(|c| {
                c.token_count() >= opts.min_tokens
                    && !(opts.skip_boilerplate && c.boilerplate)
                    && !(opts.skip_tests && c.is_test)
                    && c.lang.family() == q.lang.family()
                    && !same_place(q, c)
            })
            .map(|c| (c, score_with(q, c, &opts.weights)))
            .filter(|(c, s)| {
                s.combined >= opts.threshold
                    && s.name >= opts.min_name
                    // tiny units and module-level values only count as (near-)exact same-name copies
                    && (!(q.lines.min(c.lines) < opts.min_lines
                        || q.token_count().min(c.token_count()) < 20
                        || q.kind == "value"
                        || c.kind == "value")
                        || (s.name >= 0.8 && s.structural >= 0.95))
                    // test code only counts when the body is (nearly) identical
                    && (!(q.is_test || c.is_test) || s.structural >= 0.95)
            })
            .collect();
        hits.sort_by(|a, b| b.1.combined.total_cmp(&a.1.combined));
        hits.truncate(opts.top_n);
        hits
    }
}

/// For each query unit, the best-scoring corpus units of the same language family.
pub fn find_matches(queries: &[Unit], corpus: &Corpus, opts: MatchOptions) -> Vec<Match> {
    queries
        .iter()
        .filter(|q| {
            q.token_count() >= opts.min_tokens
                && !(opts.skip_boilerplate && q.boilerplate)
                && !(opts.skip_tests && q.is_test)
        })
        .flat_map(|q| {
            corpus.best_for(q, opts).into_iter().map(|(c, scores)| Match {
                query: q.into(),
                candidate: c.into(),
                scores,
            })
        })
        .collect()
}

/// Extract units from a single file (language from the extension); empty if unsupported/unreadable.
fn is_generated(path: &std::path::Path) -> bool {
    let p = path.to_string_lossy();
    p.contains(".gen.") || p.contains("/generated/") || p.contains("/__generated__/") || p.ends_with(".d.ts")
}

pub fn units_from_file(root: &std::path::Path, path: &std::path::Path) -> Vec<Unit> {
    if is_generated(path) {
        return Vec::new();
    }
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
    &["node_modules", "target", ".venv", "venv", "__pycache__", "dist", "build", ".codegraph", ".claude", ".worktrees"];

/// Supported source files below `root`, honouring `.gitignore`/`.ignore`, skipping hidden and
/// vendored directories, and any `excludes` globs (gitignore syntax, e.g. `**/generated/**`).
pub fn walk_files(root: &std::path::Path, excludes: &[String]) -> Vec<std::path::PathBuf> {
    let mut b = ignore::WalkBuilder::new(root);
    b.require_git(false).filter_entry(|e| {
        e.depth() == 0 || !e.file_name().to_str().is_some_and(|n| SKIP_DIRS.contains(&n))
    });
    if !excludes.is_empty() {
        let mut ob = ignore::overrides::OverrideBuilder::new(root);
        for g in excludes {
            let _ = ob.add(&format!("!{g}"));
        }
        if let Ok(o) = ob.build() {
            b.overrides(o);
        }
    }
    b.build()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_some_and(|t| t.is_file()))
        .map(|e| e.into_path())
        .filter(|p| crate::lang::Lang::from_path(p).is_some())
        .collect()
}

/// All units of all supported files below `root`.
pub fn load_units(root: &std::path::Path) -> Vec<Unit> {
    load_units_with(root, &[])
}

pub fn load_units_with(root: &std::path::Path, excludes: &[String]) -> Vec<Unit> {
    walk_files(root, excludes).iter().flat_map(|p| units_from_file(root, p)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{extract_units, Lang};

    fn units(file: &str, src: &str) -> Vec<Unit> {
        extract_units(file, Lang::Python, src)
    }

    #[test]
    fn finds_renamed_copy_and_ignores_unrelated() {
        let a = "def total(items):\n    result = 0\n    for item in items:\n        if item > 0:\n            result += item\n    return result\n";
        let b = "def summe(values):\n    acc = 0\n    for v in values:\n        if v > 0:\n            acc += v\n    return acc\n";
        let c = "def other(path):\n    with open(path) as fh:\n        return fh.read().splitlines()\n";
        let mut corpus = units("a.py", a);
        corpus.extend(units("c.py", c));
        let corpus = Corpus::new(corpus);
        let q = units("b.py", b);
        let m = find_matches(&q, &corpus, MatchOptions { min_tokens: 5, min_lines: 0, min_name: 0.0, ..Default::default() });
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].candidate.name, "total");
        assert!(m[0].scores.structural > 0.99);
    }

    #[test]
    fn index_agrees_with_brute_force_above_threshold() {
        let srcs = [
            "def f(x):\n    y = x + 1\n    z = y * 2 + x\n    w = z - y + x * 3\n    return w * 2\n",
            "def g(x):\n    y = x + 1\n    z = y * 2 + x\n    w = z - y + x * 3\n    return w * 3\n",
            "def h(p):\n    out = [i for i in p if i]\n    total = sum(out) + len(out)\n    return total\n",
        ];
        let all: Vec<Unit> = srcs.iter().enumerate().flat_map(|(i, s)| units(&format!("{i}.py"), s)).collect();
        let corpus = Corpus::new(all.clone());
        let opts = MatchOptions { threshold: 0.45, min_tokens: 3, top_n: 10, min_lines: 0, min_name: 0.0, weights: Weights::default(), skip_tests: false, skip_boilerplate: false };
        for q in &all {
            let brute = all
                .iter()
                .filter(|c| !same_place(q, c) && score(q, c).combined >= 0.45)
                .count();
            assert_eq!(corpus.best_for(q, opts).len(), brute);
        }
    }
}

/// One whole-file unit per supported file below `root` (for file-level comparisons).
pub fn load_file_units(root: &std::path::Path) -> Vec<Unit> {
    walk_files(root, &[])
        .iter()
        .filter_map(|p| {
            let lang = crate::lang::Lang::from_path(p)?;
            let source = std::fs::read_to_string(p).ok()?;
            let rel = p.strip_prefix(root).unwrap_or(p).to_string_lossy().replace('\\', "/");
            crate::units::extract_file_unit(&rel, lang, &source)
        })
        .collect()
}
