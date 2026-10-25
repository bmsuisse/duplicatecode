//! Minimal unified-diff parser: which line ranges of which (new) files were added.

use std::collections::BTreeMap;

/// new-file path -> inclusive 1-based (start, end) ranges of added lines.
pub type AddedRanges = BTreeMap<String, Vec<(u32, u32)>>;

/// Undo git's C-style quoting (`"\\303\\274.py"` for non-ASCII paths when core.quotePath is on).
fn unquote_path(p: &str) -> String {
    let Some(inner) = p.strip_prefix('"').and_then(|r| r.strip_suffix('"')) else {
        return p.to_string();
    };
    let b = inner.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'\\' || i + 1 >= b.len() {
            out.push(b[i]);
            i += 1;
            continue;
        }
        i += 1;
        match b[i] {
            b'n' => out.push(b'\n'),
            b't' => out.push(b'\t'),
            b'"' => out.push(b'"'),
            b'\\' => out.push(b'\\'),
            d @ b'0'..=b'7' => {
                let mut v = (d - b'0') as u32;
                let mut n = 1;
                while n < 3 && i + 1 < b.len() && (b'0'..=b'7').contains(&b[i + 1]) {
                    i += 1;
                    v = v * 8 + (b[i] - b'0') as u32;
                    n += 1;
                }
                out.push(v as u8);
            }
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `-a,b` / `+c,d` -> line count (default 1).
fn hunk_count(tok: &str) -> u32 {
    match tok[1..].split_once(',') {
        Some((_, n)) => n.parse().unwrap_or(1),
        None => 1,
    }
}

pub fn added_ranges(diff: &str) -> AddedRanges {
    let mut out = AddedRanges::new();
    let mut file: Option<String> = None;
    let mut new_line = 0u32;
    let mut run_start: Option<u32> = None;
    // Lines still owed by the current hunk; while > 0 a `+++ x` line is content, not a header.
    let (mut old_left, mut new_left) = (0u32, 0u32);

    let close =
        |out: &mut AddedRanges, file: &Option<String>, start: &mut Option<u32>, end: u32| {
            if let (Some(f), Some(s)) = (file, start.take()) {
                out.entry(f.clone()).or_default().push((s, end));
            }
        };

    for line in diff.lines() {
        let in_hunk = old_left > 0 || new_left > 0;
        if !in_hunk && line.starts_with("+++ ") {
            close(&mut out, &file, &mut run_start, new_line.saturating_sub(1));
            let rest = &line[4..];
            let p = if rest.starts_with('"') {
                rest
            } else {
                rest.split('\t').next().unwrap_or(rest)
            };
            let p = unquote_path(p);
            file = match p.as_str() {
                "/dev/null" => None,
                _ => Some(p.strip_prefix("b/").unwrap_or(&p).to_string()),
            };
        } else if !in_hunk && line.starts_with("@@ ") {
            close(&mut out, &file, &mut run_start, new_line.saturating_sub(1));
            // @@ -a,b +c,d @@
            let rest = &line[3..];
            let minus = rest.split_whitespace().find(|t| t.starts_with('-'));
            let plus = rest
                .split_whitespace()
                .find(|t| t.starts_with('+'))
                .unwrap_or("+1");
            let start = plus[1..].split(',').next().unwrap_or("1");
            new_line = start.parse().unwrap_or(1);
            old_left = minus.map_or(0, hunk_count);
            new_left = hunk_count(plus);
        } else if file.is_some() {
            match line.as_bytes().first() {
                Some(b'+') => {
                    run_start.get_or_insert(new_line);
                    new_line += 1;
                    new_left = new_left.saturating_sub(1);
                }
                Some(b'-') => {
                    close(&mut out, &file, &mut run_start, new_line.saturating_sub(1));
                    old_left = old_left.saturating_sub(1);
                }
                Some(b'\\') => {
                    close(&mut out, &file, &mut run_start, new_line.saturating_sub(1));
                }
                _ => {
                    close(&mut out, &file, &mut run_start, new_line.saturating_sub(1));
                    new_line += 1;
                    old_left = old_left.saturating_sub(1);
                    new_left = new_left.saturating_sub(1);
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

    #[test]
    fn quoted_non_ascii_path() {
        let d = "--- /dev/null\n+++ \"b/\\303\\274.py\"\n@@ -0,0 +1,2 @@\n+a\n+b\n";
        assert_eq!(added_ranges(d)["\u{fc}.py"], vec![(1, 2)]);
    }

    #[test]
    fn header_like_content_line_is_not_a_header() {
        let d = "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,3 @@\n a\n+++ y\n+b\n";
        let r = added_ranges(d);
        assert_eq!(r.len(), 1);
        assert_eq!(r["x.py"], vec![(2, 3)]);
    }
}
