//! Minimal unified-diff parser: which line ranges of which (new) files were added.

use std::collections::BTreeMap;

/// new-file path -> inclusive 1-based (start, end) ranges of added lines.
pub type AddedRanges = BTreeMap<String, Vec<(u32, u32)>>;

pub fn added_ranges(diff: &str) -> AddedRanges {
    let mut out = AddedRanges::new();
    let mut file: Option<String> = None;
    let mut new_line = 0u32;
    let mut run_start: Option<u32> = None;

    let close = |out: &mut AddedRanges, file: &Option<String>, start: &mut Option<u32>, end: u32| {
        if let (Some(f), Some(s)) = (file, start.take()) {
            out.entry(f.clone()).or_default().push((s, end));
        }
    };

    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("+++ ") {
            close(&mut out, &file, &mut run_start, new_line.saturating_sub(1));
            let p = rest.split('\t').next().unwrap_or(rest);
            file = match p {
                "/dev/null" => None,
                _ => Some(p.strip_prefix("b/").unwrap_or(p).to_string()),
            };
        } else if let Some(rest) = line.strip_prefix("@@ ") {
            close(&mut out, &file, &mut run_start, new_line.saturating_sub(1));
            // @@ -a,b +c,d @@
            let plus = rest.split_whitespace().find(|t| t.starts_with('+')).unwrap_or("+1");
            let start = plus[1..].split(',').next().unwrap_or("1");
            new_line = start.parse().unwrap_or(1);
        } else if file.is_some() {
            match line.as_bytes().first() {
                Some(b'+') => {
                    run_start.get_or_insert(new_line);
                    new_line += 1;
                }
                Some(b'-') | Some(b'\\') => {
                    close(&mut out, &file, &mut run_start, new_line.saturating_sub(1));
                }
                _ => {
                    close(&mut out, &file, &mut run_start, new_line.saturating_sub(1));
                    new_line += 1;
                }
            }
        }
    }
    close(&mut out, &file, &mut run_start, new_line.saturating_sub(1));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_added_ranges() {
        let d = "diff --git a/x.py b/x.py\n--- a/x.py\n+++ b/x.py\n@@ -1,3 +1,5 @@\n a\n+b\n+c\n d\n-e\n+f\n g\n";
        let r = added_ranges(d);
        assert_eq!(r["x.py"], vec![(2, 3), (5, 5)]);
    }

    #[test]
    fn new_file() {
        let d = "--- /dev/null\n+++ b/n.ts\n@@ -0,0 +1,2 @@\n+a\n+b\n";
        assert_eq!(added_ranges(d)["n.ts"], vec![(1, 2)]);
    }
}
