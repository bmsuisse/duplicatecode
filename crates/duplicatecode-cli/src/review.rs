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
    let idx: HashMap<(&str, u32), usize> =
        units.iter().enumerate().map(|(i, u)| ((u.file.as_str(), u.start_line), i)).collect();
    let matches: Vec<Match> =
        units.par_iter().flat_map_iter(|u| find_matches(std::slice::from_ref(u), &corpus, opts)).collect();

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
        let g = groups.entry(r).or_insert_with(|| G { members: vec![], edges: vec![] });
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
            g.members.sort_by(|a, b| (&units[*a].file, units[*a].start_line).cmp(&(&units[*b].file, units[*b].start_line)));
            g.edges.sort_by(|x, y| y.2.combined.total_cmp(&x.2.combined));
            let best = g.edges[0].2;
            let tier = if g.edges.iter().all(|e| e.2.structural >= 0.99 && e.2.stmt_exact >= 0.99) {
                Tier::Identical
            } else if best.combined >= 0.6 {
                Tier::NearCopy
            } else {
                Tier::Similar
            };
            (tier, best.combined, g)
        })
        .collect();
    ranked.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.total_cmp(&a.1)).then(b.2.members.len().cmp(&a.2.members.len())));

    let total = ranked.len();
    let mut out = String::new();
    out.push_str(&format!(
        "# duplicatecode review — {} units scanned, {} candidate groups (showing {})\n\n",
        units.len(),
        total,
        total.min(o.max_groups)
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
    for (i, (tier, score, g)) in ranked.iter().take(o.max_groups).enumerate() {
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
        let files: BTreeSet<&str> = us.iter().map(|u| u.file.as_str()).collect();
        if files.len() == 1 {
            hints.push("same file");
        }
        out.push_str(&format!(
            "## G{} [{}] score {:.2} · {} units{}\n",
            i + 1,
            tier.label(),
            score,
            us.len(),
            if hints.is_empty() { String::new() } else { format!(" · {}", hints.join(", ")) }
        ));
        for u in us.iter().take(8) {
            out.push_str(&format!("- {}:{}-{} {} {}\n", u.file, u.start_line, u.end_line, u.kind, u.name));
        }
        if us.len() > 8 {
            out.push_str(&format!("- … and {} more\n", us.len() - 8));
        }
        let (a, b, s) = (&units[g.edges[0].0], &units[g.edges[0].1], g.edges[0].2);
        out.push_str(&format!(
            "signals (closest pair): structure {:.2}, statements {:.2}, name {:.2}, literals {:.2}, api {:.2}\n",
            s.structural, s.stmt_exact, s.name, s.literals, s.api
        ));
        out.push_str(&format!("differs: {}\n", differences(a, b)));
        out.push_str(&preview(a, src, o.preview_lines));
        out.push('\n');
    }
    if total > o.max_groups {
        out.push_str(&format!(
            "… {} more groups not shown (raise --max-groups, or raise --threshold to narrow).\n",
            total - o.max_groups
        ));
    }
    out
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
    format!("[{}{}]", v.join(", "), if set.len() > n { ", …" } else { "" })
}

/// What distinguishes the two closest members, from cheap token-set differences.
fn differences(a: &Unit, b: &Unit) -> String {
    let mut parts = Vec::new();
    let (la, lb): (BTreeSet<_>, BTreeSet<_>) = (a.literals.iter().collect(), b.literals.iter().collect());
    let (aa, ab): (BTreeSet<_>, BTreeSet<_>) = (a.api.iter().collect(), b.api.iter().collect());
    let lit_a: BTreeSet<_> = la.difference(&lb).copied().collect();
    let lit_b: BTreeSet<_> = lb.difference(&la).copied().collect();
    let api_a: BTreeSet<_> = aa.difference(&ab).copied().collect();
    let api_b: BTreeSet<_> = ab.difference(&aa).copied().collect();
    if !lit_a.is_empty() || !lit_b.is_empty() {
        parts.push(format!("literals only in first {} / only in second {}", list(lit_a, 4), list(lit_b, 4)));
    }
    if !api_a.is_empty() || !api_b.is_empty() {
        parts.push(format!("calls/attributes only in first {} / only in second {}", list(api_a, 5), list(api_b, 5)));
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

fn preview(u: &Unit, src: &HashMap<String, PathBuf>, n: usize) -> String {
    let Some(path) = src.get(&u.file) else { return String::new() };
    let Ok(text) = std::fs::read_to_string(path) else { return String::new() };
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
