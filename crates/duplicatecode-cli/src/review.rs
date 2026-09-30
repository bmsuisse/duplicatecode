//! `review`: duplicate candidates pre-digested for an LLM (or a human) to verify.
//!
//! Compared to `scan`, the output is compact markdown: groups ranked by how likely they are real
//! duplicates, a code preview per group, what differs between the two closest members, and cheap
//! hints (tiny helper, test code, same name...) so the reader can spend its effort on judgment.

use duplicatecode_engine::index::Weights;
use duplicatecode_engine::{find_matches, Corpus, Match, MatchOptions, Unit};
use rayon::prelude::*;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::PathBuf;

pub struct ReviewOptions {
    pub threshold: f64,
    pub min_name: f64,
    pub min_lines: u32,
    pub skip_tests: bool,
    pub max_groups: usize,
    pub offset: usize,
    /// One line per group instead of the detailed block (to scan hundreds of groups cheaply).
    pub brief: bool,
    /// Only this tier (`identical`, `near`, `similar`).
    pub tier: Option<String>,
    pub preview_lines: usize,
    pub weights: Weights,
}

struct G {
    members: Vec<usize>,
    edges: Vec<(usize, usize, duplicatecode_engine::index::Scores)>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Tier {
    Identical,
    NearCopy,
    Similar,
}

impl Tier {
    fn label(self) -> &'static str {
        match self {
            Tier::Identical => "IDENTICAL",
            Tier::NearCopy => "NEAR-COPY",
            Tier::Similar => "SIMILAR",
        }
    }
}

fn find(p: &mut Vec<usize>, x: usize) -> usize {
    if p[x] != x {
        let r = find(p, p[x]);
        p[x] = r;
    }
    p[x]
}

pub fn run(units: Vec<Unit>, src: &HashMap<String, PathBuf>, o: &ReviewOptions) -> String {
    let corpus = Corpus::new(units.clone());
    let opts = MatchOptions {
        threshold: o.threshold,
        top_n: 4,
        min_name: o.min_name,
        min_lines: o.min_lines,
        skip_tests: o.skip_tests,
        weights: o.weights,
        ..Default::default()
    };
    let idx: HashMap<(&str, u32), usize> = units
        .iter()
        .enumerate()
        .map(|(i, u)| ((u.file.as_str(), u.start_line), i))
        .collect();
    let matches: Vec<Match> = units
        .par_iter()
        .flat_map_iter(|u| find_matches(std::slice::from_ref(u), &corpus, opts))
        .collect();

    // pairs -> unique edges -> connected components
    let mut seen = HashSet::new();
    let mut edges = Vec::new();
    for m in &matches {
        let (Some(&a), Some(&b)) = (
            idx.get(&(m.query.file.as_str(), m.query.start_line)),
            idx.get(&(m.candidate.file.as_str(), m.candidate.start_line)),
        ) else {
            continue;
        };
        if seen.insert((a.min(b), a.max(b))) {
            edges.push((a, b, m.scores));
        }
    }
    let mut parent: Vec<usize> = (0..units.len()).collect();
    for (a, b, _) in &edges {
        let (ra, rb) = (find(&mut parent, *a), find(&mut parent, *b));
        parent[ra] = rb;
    }
    let mut groups: HashMap<usize, G> = HashMap::new();
    for (a, b, s) in edges {
        let r = find(&mut parent, a);
        let g = groups.entry(r).or_insert_with(|| G {
            members: vec![],
            edges: vec![],
        });
        for x in [a, b] {
            if !g.members.contains(&x) {
                g.members.push(x);
            }
        }
        g.edges.push((a, b, s));
    }

    let mut ranked: Vec<(Tier, f64, G)> = groups
        .into_values()
        .map(|mut g| {
            g.members.sort_by(|a, b| {
                (&units[*a].file, units[*a].start_line)
                    .cmp(&(&units[*b].file, units[*b].start_line))
            });
            g.edges
                .sort_by(|x, y| y.2.combined.total_cmp(&x.2.combined));
            let best = g.edges[0].2;
            let tier = if g
                .edges
                .iter()
                .all(|e| e.2.structural >= 0.99 && e.2.stmt_exact >= 0.99)
            {
                Tier::Identical
            } else if best.combined >= 0.6 {
                Tier::NearCopy
            } else {
                Tier::Similar
            };
            let noisy = {
                let us: Vec<&Unit> = g.members.iter().map(|&m| &units[m]).collect();
                noise_reason(&us, &best).is_some()
            };
            (tier, best.combined - if noisy { 10.0 } else { 0.0 }, g)
        })
        .collect();
    ranked.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(b.1.total_cmp(&a.1))
            .then(b.2.members.len().cmp(&a.2.members.len()))
    });

    if let Some(t) = &o.tier {
        let want = match t.as_str() {
            "identical" => Tier::Identical,
            "near" => Tier::NearCopy,
            _ => Tier::Similar,
        };
        ranked.retain(|r| r.0 == want);
    }
    let total = ranked.len();
    let mut out = String::new();
    out.push_str(&format!(
        "# duplicatecode review — {} units scanned, {} candidate groups (showing {})\n\n",
        units.len(),
        total,
        total.saturating_sub(o.offset).min(o.max_groups)
    ));
    out.push_str(
        "Tiers: IDENTICAL = same normalized body (renames/comments/logging ignored); NEAR-COPY = very similar; \
         SIMILAR = weaker candidate. Static hints only — verify by reading the code before reporting.\n\n",
    );
    let counts = |t: Tier| ranked.iter().filter(|r| r.0 == t).count();
    out.push_str(&format!(
        "Groups: {} identical, {} near-copy, {} similar.\n\n",
        counts(Tier::Identical),
        counts(Tier::NearCopy),
        counts(Tier::Similar)
    ));
    for (i, (tier, raw_score, g)) in ranked.iter().enumerate().skip(o.offset).take(o.max_groups) {
        let score = &(if *raw_score < -5.0 {
            raw_score + 10.0
        } else {
            *raw_score
        });
        let us: Vec<&Unit> = g.members.iter().map(|&m| &units[m]).collect();
        let same_name = us.iter().all(|u| u.name == us[0].name);
        let mut hints: Vec<&str> = Vec::new();
        if us.iter().map(|u| u.lines).max().unwrap_or(0) <= 5 {
            hints.push("tiny helper");
        }
        if us.iter().all(|u| u.is_test) {
            hints.push("test code");
        } else if us.iter().any(|u| u.is_test) {
            hints.push("test + non-test");
        }
        if us.iter().any(|u| u.kind == "value") {
            hints.push("module-level value");
        }
        if same_name {
            hints.push("same name");
        }
        let noise = noise_reason(&us, &g.edges[0].2);
        if noise.is_some() {
            hints.push("LIKELY-NOISE");
        }
        let files: BTreeSet<&str> = us.iter().map(|u| u.file.as_str()).collect();
        if files.len() == 1 {
            hints.push("same file");
        }
        if o.brief {
            let mem: Vec<String> = us
                .iter()
                .take(3)
                .map(|u| format!("{}:{}-{}", u.file, u.start_line, u.end_line))
                .collect();
            out.push_str(&format!(
                "G{} {} {:.2} x{} {}{}: {}{}\n",
                i + 1,
                tier.label(),
                score,
                us.len(),
                if same_name {
                    format!("`{}` ", us[0].name)
                } else {
                    format!("`{}`~`{}` ", us[0].name, us[1].name)
                },
                if hints.is_empty() {
                    String::new()
                } else {
                    format!("[{}]", hints.join(","))
                },
                mem.join(" | "),
                if us.len() > 3 {
                    format!(" | +{}", us.len() - 3)
                } else {
                    String::new()
                }
            ));
            continue;
        }
        out.push_str(&format!(
            "## G{} [{}] score {:.2} · {} units{}\n",
            i + 1,
            tier.label(),
            score,
            us.len(),
            if hints.is_empty() {
                String::new()
            } else {
                format!(" · {}", hints.join(", "))
            }
        ));
        for u in us.iter().take(8) {
            out.push_str(&format!(
                "- {}:{}-{} {} {}\n",
                u.file, u.start_line, u.end_line, u.kind, u.name
            ));
        }
        if us.len() > 8 {
            out.push_str(&format!("- … and {} more\n", us.len() - 8));
        }
        let (a, b, s) = (&units[g.edges[0].0], &units[g.edges[0].1], g.edges[0].2);
        out.push_str(&format!(
            "signals (closest pair): structure {:.2}, statements {:.2}, name {:.2}, literals {:.2}, api {:.2}\n",
            s.structural, s.stmt_exact, s.name, s.literals, s.api
        ));
        if let Some(n) = noise {
            out.push_str(&format!("likely noise: {n}\n"));
        }
        out.push_str(&format!("differs: {}\n", differences(a, b)));
        out.push_str(&diff_view(a, b, src));
        out.push_str(&preview(a, src, o.preview_lines));
        out.push('\n');
    }
    if total > o.offset + o.max_groups {
        out.push_str(&format!(
            "… {} more groups not shown: continue with `--offset {}` (add `--brief` for one line per group; `--tier identical|near|similar` filters).\n",
            total - o.offset - o.max_groups,
            o.offset + o.max_groups
        ));
    }
    out
}

/// Cheap pattern detection for the top false-alarm families, so a reader can skip them quickly.
fn noise_reason(us: &[&Unit], best: &duplicatecode_engine::index::Scores) -> Option<&'static str> {
    let names_differ = best.name < 0.8;
    let all_le = |n: u32| us.iter().all(|u| u.lines <= n);
    if names_differ && best.structural >= 0.95 && best.api >= 0.99 && all_le(15) {
        return Some("parametrized twin: same shape and calls, differs only in names/literals (likely per-entity wrapper)");
    }
    if names_differ
        && us
            .iter()
            .all(|u| u.shape_seq.len() <= 3 && u.callees.len() <= 2)
    {
        return Some("thin delegator: few statements, one or two calls");
    }
    None
}

fn trunc(s: &str) -> String {
    if s.chars().count() > 22 {
        format!("{}…", s.chars().take(22).collect::<String>())
    } else {
        s.to_string()
    }
}

fn list(set: BTreeSet<&String>, n: usize) -> String {
    let v: Vec<String> = set.iter().take(n).map(|s| trunc(s)).collect();
    format!(
        "[{}{}]",
        v.join(", "),
        if set.len() > n { ", …" } else { "" }
    )
}

/// What distinguishes the two closest members, from cheap token-set differences.
fn differences(a: &Unit, b: &Unit) -> String {
    let mut parts = Vec::new();
    let (la, lb): (BTreeSet<_>, BTreeSet<_>) =
        (a.literals.iter().collect(), b.literals.iter().collect());
    let (aa, ab): (BTreeSet<_>, BTreeSet<_>) = (a.api.iter().collect(), b.api.iter().collect());
    let lit_a: BTreeSet<_> = la.difference(&lb).copied().collect();
    let lit_b: BTreeSet<_> = lb.difference(&la).copied().collect();
    let api_a: BTreeSet<_> = aa.difference(&ab).copied().collect();
    let api_b: BTreeSet<_> = ab.difference(&aa).copied().collect();
    if !lit_a.is_empty() || !lit_b.is_empty() {
        parts.push(format!(
            "literals only in first {} / only in second {}",
            list(lit_a, 4),
            list(lit_b, 4)
        ));
    }
    if !api_a.is_empty() || !api_b.is_empty() {
        parts.push(format!(
            "calls/attributes only in first {} / only in second {}",
            list(api_a, 5),
            list(api_b, 5)
        ));
    }
    if a.lines != b.lines {
        parts.push(format!("length {} vs {} lines", a.lines, b.lines));
    }
    if parts.is_empty() {
        "nothing detectable (identical after normalization)".to_string()
    } else {
        parts.join("; ")
    }
}

/// Line diff of the two closest members (changed lines with 1 line of context), so a reader can
/// judge "copy with small edits" vs "different code" without opening the files.
fn diff_view(a: &Unit, b: &Unit, src: &HashMap<String, PathBuf>) -> String {
    let text = |u: &Unit| -> Option<String> {
        let t = std::fs::read_to_string(src.get(&u.file)?).ok()?;
        let lines: Vec<&str> = t.lines().collect();
        let (s, e) = (
            (u.start_line as usize).saturating_sub(1),
            (u.end_line as usize).min(lines.len()),
        );
        Some(
            lines[s.min(e)..e]
                .iter()
                .map(|l| l.trim_end().to_string() + "\n")
                .collect(),
        )
    };
    let (Some(ta), Some(tb)) = (text(a), text(b)) else {
        return String::new();
    };
    let d = similar::TextDiff::configure()
        .algorithm(similar::Algorithm::Myers)
        .diff_lines(&ta, &tb);
    let ratio = d.ratio();
    if ratio >= 0.9999 {
        return "diff (first vs second): textually identical\n".to_string();
    }
    let mut out = format!(
        "diff (first vs second, {:.0}% of lines equal):\n```diff\n",
        ratio * 100.0
    );
    let mut shown = 0;
    'outer: for group in d.grouped_ops(1) {
        for op in group {
            for ch in d.iter_changes(&op) {
                let sign = match ch.tag() {
                    similar::ChangeTag::Delete => '-',
                    similar::ChangeTag::Insert => '+',
                    similar::ChangeTag::Equal => ' ',
                };
                let l: String = ch.value().trim_end().chars().take(100).collect();
                out.push_str(&format!("{sign}{l}\n"));
                shown += 1;
                if shown >= 22 {
                    out.push_str("… (diff truncated)\n");
                    break 'outer;
                }
            }
        }
        out.push_str("…\n");
    }
    out.push_str("```\n");
    out
}

fn preview(u: &Unit, src: &HashMap<String, PathBuf>, n: usize) -> String {
    let Some(path) = src.get(&u.file) else {
        return String::new();
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return String::new();
    };
    let lines: Vec<&str> = text.lines().collect();
    let start = (u.start_line as usize).saturating_sub(1);
    let end = (u.end_line as usize).min(lines.len());
    let mut s = format!("preview of {}:\n```\n", u.name);
    for (i, l) in lines[start.min(end)..end].iter().take(n).enumerate() {
        let l: String = l.chars().take(110).collect();
        s.push_str(&format!("{:>5}| {}\n", start + i + 1, l));
    }
    if end - start.min(end) > n {
        s.push_str("      | … \n");
    }
    s.push_str("```\n");
    s
}
