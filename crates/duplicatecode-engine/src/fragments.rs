//! Fragment-level clones: a run of identical normalized statements shared by two different units
//! (a block pasted into another function), which whole-unit scoring dilutes or misses.
//!
//! Every window of `min_stmts` consecutive statement hashes is indexed; units sharing a window are
//! paired, and windows on the same diagonal are merged into maximal shared runs.

use crate::units::Unit;
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Copy, Debug)]
pub struct FragmentOptions {
    /// Minimum number of consecutive identical statements.
    pub min_stmts: usize,
    /// Minimum normalized tokens in the shared run (skips runs of trivial statements).
    pub min_tokens: usize,
    /// Windows occurring in more units than this are boilerplate and ignored.
    pub max_postings: usize,
    /// Statements that may be inserted, deleted or edited between two shared runs of the same
    /// pair without splitting the fragment (a pasted block that was adapted a little).
    pub max_gap: usize,
}

impl Default for FragmentOptions {
    fn default() -> Self {
        FragmentOptions {
            min_stmts: 4,
            min_tokens: 30,
            max_postings: 64,
            max_gap: 1,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct FragmentRef {
    pub file: String,
    pub unit: String,
    pub start_line: u32,
    pub end_line: u32,
    /// Share of the unit's statements covered by the fragment.
    pub coverage: f64,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Fragment {
    pub a: FragmentRef,
    pub b: FragmentRef,
    pub statements: usize,
    pub tokens: usize,
}

fn window_key(hashes: &[u64]) -> u64 {
    hashes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, x| {
        (h ^ x).wrapping_mul(0x0100_0000_01b3).rotate_left(7)
    })
}

/// One unit lies inside the other (class and its method, nested functions): not a clone.
fn nested(a: &Unit, b: &Unit) -> bool {
    a.file == b.file
        && ((a.start_line <= b.start_line && b.end_line <= a.end_line)
            || (b.start_line <= a.start_line && a.end_line <= b.end_line))
}

pub fn find_fragments(units: &[Unit], opts: FragmentOptions) -> Vec<Fragment> {
    let k = opts.min_stmts.max(2);
    let mut postings: HashMap<u64, Vec<(u32, u32)>> = HashMap::new();
    for (ui, u) in units.iter().enumerate() {
        if u.frag.len() < k {
            continue;
        }
        let hashes: Vec<u64> = u.frag.iter().map(|s| s.hash).collect();
        for (pos, w) in hashes.windows(k).enumerate() {
            postings
                .entry(window_key(w))
                .or_default()
                .push((ui as u32, pos as u32));
        }
    }
    // (unit a, unit b, diagonal) -> positions of shared windows in a
    let mut diagonals: BTreeMap<(u32, u32, i64), Vec<u32>> = BTreeMap::new();
    for list in postings.values().filter(|l| l.len() <= opts.max_postings) {
        for (i, &(ua, pa)) in list.iter().enumerate() {
            for &(ub, pb) in &list[i + 1..] {
                if ua == ub || nested(&units[ua as usize], &units[ub as usize]) {
                    continue;
                }
                let (lo, hi, plo, phi) = if ua < ub {
                    (ua, ub, pa, pb)
                } else {
                    (ub, ua, pb, pa)
                };
                diagonals
                    .entry((lo, hi, plo as i64 - phi as i64))
                    .or_default()
                    .push(plo);
            }
        }
    }
    // maximal exact runs per diagonal: (unit a, unit b) -> (start in a, start in b, length)
    let mut runs: BTreeMap<(u32, u32), Vec<(usize, usize, usize)>> = BTreeMap::new();
    for ((ua, ub, diag), mut positions) in diagonals {
        positions.sort_unstable();
        positions.dedup();
        let mut i = 0;
        while i < positions.len() {
            let mut j = i;
            while j + 1 < positions.len() && positions[j + 1] == positions[j] + 1 {
                j += 1;
            }
            let first = positions[i] as usize;
            let len = positions[j] as usize - first + k;
            runs.entry((ua, ub))
                .or_default()
                .push((first, (first as i64 - diag) as usize, len));
            i = j + 1;
        }
    }
    let mut out = Vec::new();
    for ((ua, ub), mut rs) in runs {
        let (a, b) = (&units[ua as usize], &units[ub as usize]);
        rs.sort_unstable();
        // chain runs whose gap is at most `max_gap` statements on both sides: an inserted,
        // deleted or edited statement inside a pasted block must not split it
        let mut consumed = vec![false; rs.len()];
        for i in 0..rs.len() {
            if consumed[i] {
                continue;
            }
            consumed[i] = true;
            let mut chain = vec![rs[i]];
            for j in i + 1..rs.len() {
                if consumed[j] {
                    continue;
                }
                let &(la, lb, ln) = chain.last().unwrap();
                let (na, nb, _) = rs[j];
                if na > la + ln + opts.max_gap {
                    break;
                }
                if na >= la + ln && nb >= lb + ln && nb - (lb + ln) <= opts.max_gap {
                    consumed[j] = true;
                    chain.push(rs[j]);
                }
            }
            let last = *chain.last().unwrap();
            let first = chain[0];
            let (span_a, span_b) = (
                &a.frag[first.0..last.0 + last.2],
                &b.frag[first.1..last.1 + last.2],
            );
            let stmts_a: Vec<_> = chain
                .iter()
                .flat_map(|r| a.frag[r.0..r.0 + r.2].iter())
                .collect();
            let tokens: usize = stmts_a.iter().map(|s| s.tokens as usize).sum();
            // a comprehension is expanded into several pseudo-statements on one source line:
            // count source statements, not entries
            let source_stmts = stmts_a
                .iter()
                .map(|s| (s.start, s.end))
                .collect::<std::collections::BTreeSet<_>>()
                .len();
            let span_len = last.0 + last.2 - first.0;
            if tokens < opts.min_tokens || source_stmts < k {
                continue;
            }
            let side = |u: &Unit, s: &[crate::units::FragStmt]| FragmentRef {
                file: u.file.clone(),
                unit: u.name.clone(),
                start_line: s.first().map_or(0, |x| x.start),
                end_line: s.last().map_or(0, |x| x.end),
                coverage: span_len as f64 / u.frag.len().max(1) as f64,
            };
            out.push(Fragment {
                a: side(a, span_a),
                b: side(b, span_b),
                statements: source_stmts,
                tokens,
            });
        }
    }
    out.sort_by_key(|f| std::cmp::Reverse(f.tokens));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{extract_units, Lang};

    #[test]
    fn finds_a_pasted_block_with_renamed_variables() {
        let a = "def process(rows):\n    total = 0\n    count = 0\n    for r in rows:\n        if r.ok:\n            total += r.value\n            count += 1\n    avg = total / count\n    print(avg)\n    return avg\n";
        let b = "def other(path, flag):\n    log = open(path)\n    n = 0\n    s = 0\n    for item in log:\n        if item.ok:\n            s += item.value\n            n += 1\n    mean = s / n\n    if flag:\n        close(log)\n    return mean\n";
        let c = "def unrelated(x):\n    y = x + 1\n    z = y * 2\n    w = z - y\n    v = w + x\n    u = v * 3\n    return u\n";
        let mut units = extract_units("a.py", Lang::Python, a);
        units.extend(extract_units("b.py", Lang::Python, b));
        units.extend(extract_units("c.py", Lang::Python, c));
        let found = find_fragments(
            &units,
            FragmentOptions {
                min_stmts: 4,
                min_tokens: 20,
                ..Default::default()
            },
        );
        assert_eq!(found.len(), 1, "{found:#?}");
        let f = &found[0];
        assert!(f.statements >= 4);
        assert_eq!((f.a.file.as_str(), f.b.file.as_str()), ("a.py", "b.py"));
        assert!(f.a.coverage < 0.9 && f.b.coverage < 0.9);
    }

    #[test]
    fn a_repeated_one_line_comprehension_is_not_a_block() {
        let a = "def f(xs):\n    return [x * 2 + 1 for x in xs if x > 0 and x < 100]\n";
        let b = "def g(ys):\n    return [y * 2 + 1 for y in ys if y > 0 and y < 100]\n";
        let mut units = extract_units("a.py", Lang::Python, a);
        units.extend(extract_units("b.py", Lang::Python, b));
        let opts = FragmentOptions {
            min_stmts: 4,
            min_tokens: 5,
            ..Default::default()
        };
        assert!(find_fragments(&units, opts).is_empty());
    }

    #[test]
    fn nested_units_are_not_reported() {
        let src = "class K:\n    def m(self, x):\n        a = x + 1\n        b = a * 2\n        c = b - a\n        d = c + x\n        e = d * 3\n        return e\n";
        let units = extract_units("k.py", Lang::Python, src);
        assert!(find_fragments(
            &units,
            FragmentOptions {
                min_stmts: 3,
                min_tokens: 5,
                ..Default::default()
            }
        )
        .is_empty());
    }

    #[test]
    fn an_edited_statement_inside_a_pasted_block_does_not_split_it() {
        let a = "def process(rows):\n    total = 0\n    count = 0\n    for r in rows:\n        if r.ok:\n            total += r.value\n            count += 1\n    avg = total / count\n    print(avg)\n    return avg\n";
        // one statement edited in the middle of the block
        let b = "def other(path):\n    n = 0\n    s = 0\n    for item in path:\n        if item.ok:\n            s += item.value * 2\n            n += 1\n    mean = s / n\n    print(mean)\n    return mean\n";
        let mut units = extract_units("a.py", Lang::Python, a);
        units.extend(extract_units("b.py", Lang::Python, b));
        let run = |max_gap| {
            find_fragments(
                &units,
                FragmentOptions {
                    min_stmts: 3,
                    min_tokens: 10,
                    max_gap,
                    ..Default::default()
                },
            )
        };
        let tolerant = run(1);
        let strict = run(0);
        let best = |v: &[Fragment]| {
            v.iter()
                .map(|f| f.a.end_line - f.a.start_line)
                .max()
                .unwrap_or(0)
        };
        assert!(!tolerant.is_empty(), "{tolerant:#?}");
        assert!(
            best(&tolerant) > best(&strict),
            "{tolerant:#?} vs {strict:#?}"
        );
    }
}
