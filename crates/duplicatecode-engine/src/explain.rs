//! What differs between two similar units: the literals, calls and statements to parameterize when
//! extracting a shared helper.

use crate::units::Unit;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Explanation {
    pub literals_only_a: Vec<String>,
    pub literals_only_b: Vec<String>,
    pub calls_only_a: Vec<String>,
    pub calls_only_b: Vec<String>,
    /// Source line ranges of statements with no counterpart in the other unit.
    pub lines_only_a: Vec<(u32, u32)>,
    pub lines_only_b: Vec<(u32, u32)>,
    pub shared_statements: usize,
    pub statements_a: usize,
    pub statements_b: usize,
}

/// Statements of `x` that `y` does not also contain (multiset difference by normalized hash),
/// merged into line ranges.
fn unmatched(x: &Unit, y: &Unit) -> (Vec<(u32, u32)>, usize) {
    let mut avail: HashMap<u64, u32> = HashMap::new();
    for s in &y.frag {
        *avail.entry(s.hash).or_default() += 1;
    }
    let (mut ranges, mut shared): (Vec<(u32, u32)>, usize) = (Vec::new(), 0);
    for s in &x.frag {
        match avail.get_mut(&s.hash).filter(|n| **n > 0) {
            Some(n) => {
                *n -= 1;
                shared += 1;
            }
            None => match ranges.last_mut() {
                Some(r) if s.start <= r.1 + 1 => r.1 = r.1.max(s.end),
                _ => ranges.push((s.start, s.end)),
            },
        }
    }
    (ranges, shared)
}

pub fn explain(a: &Unit, b: &Unit) -> Explanation {
    let only = |x: &std::collections::BTreeSet<String>, y: &std::collections::BTreeSet<String>| {
        x.difference(y).take(12).cloned().collect::<Vec<_>>()
    };
    let (lines_only_a, shared) = unmatched(a, b);
    let (lines_only_b, _) = unmatched(b, a);
    Explanation {
        literals_only_a: only(&a.literals, &b.literals),
        literals_only_b: only(&b.literals, &a.literals),
        calls_only_a: only(&a.callees, &b.callees),
        calls_only_b: only(&b.callees, &a.callees),
        lines_only_a,
        lines_only_b,
        shared_statements: shared,
        statements_a: a.frag.len(),
        statements_b: b.frag.len(),
    }
}

impl Explanation {
    /// One-line human summary, empty when the units are indistinguishable.
    pub fn summary(&self) -> String {
        let set = |v: &[String]| v.join(", ");
        let ranges = |v: &[(u32, u32)]| {
            v.iter()
                .map(|(s, e)| {
                    if s == e {
                        s.to_string()
                    } else {
                        format!("{s}-{e}")
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        let mut parts = Vec::new();
        if !self.literals_only_a.is_empty() || !self.literals_only_b.is_empty() {
            parts.push(format!(
                "literals A[{}] B[{}]",
                set(&self.literals_only_a),
                set(&self.literals_only_b)
            ));
        }
        if !self.calls_only_a.is_empty() || !self.calls_only_b.is_empty() {
            parts.push(format!(
                "calls A[{}] B[{}]",
                set(&self.calls_only_a),
                set(&self.calls_only_b)
            ));
        }
        if !self.lines_only_a.is_empty() || !self.lines_only_b.is_empty() {
            parts.push(format!(
                "differing lines A[{}] B[{}] ({} of {}/{} statements shared)",
                ranges(&self.lines_only_a),
                ranges(&self.lines_only_b),
                self.shared_statements,
                self.statements_a,
                self.statements_b
            ));
        }
        parts.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{extract_units, Lang};

    #[test]
    fn names_the_literals_calls_and_lines_that_differ() {
        let a =
            "def f(x):\n    y = x + 1\n    log('start')\n    z = fetch(y, 10)\n    return z * 2\n";
        let b =
            "def g(x):\n    y = x + 1\n    log('start')\n    z = load(y, 20)\n    return z * 2\n";
        let ua = extract_units("a.py", Lang::Python, a).remove(0);
        let ub = extract_units("b.py", Lang::Python, b).remove(0);
        let e = explain(&ua, &ub);
        assert_eq!(e.literals_only_a, vec!["10"]);
        assert_eq!(e.literals_only_b, vec!["20"]);
        assert_eq!(e.calls_only_a, vec!["fetch"]);
        assert_eq!(e.calls_only_b, vec!["load"]);
        assert!(e.shared_statements >= 2);
        assert!(!e.summary().is_empty());
        assert!(explain(&ua, &ua).summary().is_empty());
    }
}
