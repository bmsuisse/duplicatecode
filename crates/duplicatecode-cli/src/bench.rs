//! Benchmark: do independent implementations of the same task find each other?
//!
//! Ground truth is the task number. For every query unit in set A, all units of set B (same
//! language family) are ranked by a scorer; the ranking is correct if the top hit belongs to the
//! same task.

use anyhow::{Context, Result};
use duplicatecode_engine::index::{score, Scores};
use duplicatecode_engine::{load_units, Corpus, MatchOptions, Unit};
use std::collections::HashMap;
use std::path::Path;

struct Item {
    group: String,
    unit: Unit,
}

/// combo name ("strict/haiku") -> items
type Combos = Vec<(String, Vec<Item>)>;

/// strict target file name -> task number, from the `**Target file:** \`x\`` line of each spec.
fn strict_targets(dataset: &Path) -> Result<HashMap<String, String>> {
    let mut m = HashMap::new();
    for e in std::fs::read_dir(dataset.join("tasks"))? {
        let p = e?.path();
        if p.extension().is_none_or(|x| x != "md") {
            continue;
        }
        let nn = p.file_name().unwrap().to_string_lossy()[..2].to_string();
        let text = std::fs::read_to_string(&p)?;
        let target = text
            .lines()
            .find(|l| l.contains("Target file:"))
            .and_then(|l| l.split('`').nth(1))
            .with_context(|| format!("no target file in {}", p.display()))?;
        m.insert(target.to_string(), nn);
    }
    Ok(m)
}

fn load_combos(dataset: &Path) -> Result<Combos> {
    let targets = strict_targets(dataset)?;
    let mut combos = Combos::new();
    for (set, dir) in [("strict", "impls"), ("loose", "impls-loose"), ("hard", "impls-hard")] {
        let Ok(rd) = std::fs::read_dir(dataset.join(dir)) else { continue };
        let mut models: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.is_dir()).collect();
        models.sort();
        for mdir in models {
            let model = mdir.file_name().unwrap().to_string_lossy().to_string();
            let mut items = Vec::new();
            for u in load_units(&mdir) {
                let group = if set == "strict" {
                    let base = u.file.rsplit('/').next().unwrap_or(&u.file);
                    targets.get(base).cloned()
                } else {
                    Some(u.file[..2].to_string())
                };
                if let Some(group) = group {
                    items.push(Item { group, unit: u });
                }
            }
            combos.push((format!("{set}/{model}"), items));
        }
    }
    Ok(combos)
}

type Pick = fn(&Scores) -> f64;
const SCORERS: [(&str, Pick); 5] = [
    ("structure", |s| s.structural),
    ("contain", |s| s.containment),
    ("name", |s| s.name),
    ("callees", |s| s.callees),
    ("combined", |s| s.combined),
];

pub fn run(dataset: &Path, min_tokens: usize, negatives: Option<&Path>) -> Result<()> {
    let combos = load_combos(dataset)?;
    for (name, items) in &combos {
        println!("loaded {name}: {} units", items.len());
    }

    // (category label, predicate over (query combo, candidate combo))
    let categories: [(&str, fn(&str, &str) -> bool); 6] = [
        ("strict~strict (cross-model)", |a, b| pair(a, b, "strict", "strict") && a != b),
        ("loose~loose  (cross-model)", |a, b| pair(a, b, "loose", "loose") && a != b),
        ("hard~hard    (cross-model)", |a, b| pair(a, b, "hard", "hard") && a != b),
        ("strict~loose (cross-model)", |a, b| pair(a, b, "strict", "loose") && model(a) != model(b)),
        ("strict~hard  (cross-model)", |a, b| pair(a, b, "strict", "hard") && model(a) != model(b)),
        ("loose~hard   (cross-model)", |a, b| pair(a, b, "loose", "hard") && model(a) != model(b)),
    ];

    for family in ["python", "typescript"] {
        println!("\n== {family}: top-1 accuracy (queries) ==");
        print!("{:<30}", "");
        for (n, _) in SCORERS {
            print!("{n:>10}");
        }
        println!("{:>12}", "combined@3");
        for (label, pred) in categories {
            let mut correct = [0usize; SCORERS.len()];
            let mut top3 = 0usize;
            let mut total = 0usize;
            for (an, aitems) in &combos {
                for (bn, bitems) in &combos {
                    if !pred(an, bn) {
                        continue;
                    }
                    for q in aitems.iter().filter(|i| i.unit.lang.family() == family && i.unit.token_count() >= min_tokens) {
                        let cands: Vec<&Item> = bitems
                            .iter()
                            .filter(|c| c.unit.lang.family() == family && c.unit.token_count() >= min_tokens)
                            .collect();
                        if cands.is_empty() {
                            continue;
                        }
                        total += 1;
                        let scored: Vec<(&Item, Scores)> =
                            cands.iter().map(|c| (*c, score(&q.unit, &c.unit))).collect();
                        for (k, (_, pick)) in SCORERS.iter().enumerate() {
                            let best = scored.iter().max_by(|x, y| pick(&x.1).total_cmp(&pick(&y.1))).unwrap();
                            if best.0.group == q.group {
                                correct[k] += 1;
                            }
                        }
                        let mut ranked: Vec<&(&Item, Scores)> = scored.iter().collect();
                        ranked.sort_by(|x, y| y.1.combined.total_cmp(&x.1.combined));
                        if ranked.iter().take(3).any(|x| x.0.group == q.group) {
                            top3 += 1;
                        }
                    }
                }
            }
            print!("{label:<30}");
            for c in correct {
                print!("{:>9.0}%", 100.0 * c as f64 / total.max(1) as f64);
            }
            println!("{:>11.0}%   (n={total})", 100.0 * top3 as f64 / total.max(1) as f64);
        }
    }
    if let Some(neg) = negatives {
        sweep(&combos, &categories, neg, min_tokens);
    }
    Ok(())
}

/// Threshold sweep: recall on true (same-task, cross-model) pairs vs. false alarms against an
/// unrelated corpus. Also prints the highest-scoring corpus hits for manual inspection.
fn sweep(combos: &Combos, categories: &[(&str, fn(&str, &str) -> bool)], neg: &Path, min_tokens: usize) {
    use rayon::prelude::*;
    let corpus = Corpus::new(load_units(neg));
    let opts = MatchOptions { threshold: 0.0, min_tokens, top_n: 1 };
    println!("\n== threshold sweep vs. negatives corpus {} ({} units) ==", neg.display(), corpus.units().len());

    // best combined score per dataset unit against the negatives (+ which corpus unit)
    let mut best: HashMap<(String, String, u32), (f64, String, &'static str)> = HashMap::new();
    let queries: Vec<(&String, &Item)> = combos
        .iter()
        .flat_map(|(n, items)| items.iter().filter(|i| i.unit.token_count() >= min_tokens).map(move |i| (n, i)))
        .collect();
    let results: Vec<_> = queries
        .par_iter()
        .map(|(n, q)| {
            let (top, who) = match corpus.best_for(&q.unit, opts).first() {
                Some((c, sc)) => (sc.combined, format!("{}:{} {}", c.file, c.start_line, c.name)),
                None => (0.0, String::new()),
            };
            ((n.to_string(), q.unit.file.clone(), q.unit.start_line), (top, who, q.unit.lang.family()))
        })
        .collect();
    best.extend(results);

    let mut worst: Vec<_> = best.iter().collect();
    worst.sort_by(|a, b| b.1 .0.total_cmp(&a.1 .0));
    println!("highest-scoring negatives (inspect by hand: real duplicates or false alarms?):");
    for ((combo, file, line), (s, who, _)) in worst.iter().take(12) {
        println!("  {s:.2}  {combo} {file}:{line}  ~  {who}");
    }

    let thresholds = [0.45, 0.5, 0.55, 0.6, 0.7, 0.8];
    print!("\n{:<34}", "recall of true pairs @ threshold");
    for t in thresholds {
        print!("{t:>7.2}");
    }
    println!();
    for family in ["python", "typescript"] {
        for (label, pred) in categories {
            let mut hits = [0usize; 6];
            let mut total = 0usize;
            for (an, aitems) in combos {
                for (bn, bitems) in combos {
                    if !pred(an, bn) {
                        continue;
                    }
                    for q in aitems.iter().filter(|i| i.unit.lang.family() == family && i.unit.token_count() >= min_tokens) {
                        let best_true = bitems
                            .iter()
                            .filter(|c| c.group == q.group && c.unit.token_count() >= min_tokens)
                            .map(|c| score(&q.unit, &c.unit).combined)
                            .fold(f64::NAN, f64::max);
                        if best_true.is_nan() {
                            continue;
                        }
                        total += 1;
                        for (k, t) in thresholds.iter().enumerate() {
                            hits[k] += (best_true >= *t) as usize;
                        }
                    }
                }
            }
            print!("{:<34}", format!("{family} {label}"));
            for h in hits {
                print!("{:>6.0}%", 100.0 * h as f64 / total.max(1) as f64);
            }
            println!();
        }
        let items: Vec<_> = best.values().filter(|v| v.2 == family).collect();
        let n = items.len().max(1);
        print!("{:<34}", format!("{family} FALSE-ALARM rate (queries)"));
        for t in thresholds {
            print!("{:>6.0}%", 100.0 * items.iter().filter(|v| v.0 >= t).count() as f64 / n as f64);
        }
        println!("  (n={})", items.len());
    }
}

/// Directional: queries come from set `x`, candidates from set `y`.
fn pair(a: &str, b: &str, x: &str, y: &str) -> bool {
    kind(a) == x && kind(b) == y
}

fn kind(combo: &str) -> &str {
    combo.split('/').next().unwrap()
}
fn model(combo: &str) -> &str {
    combo.split('/').nth(1).unwrap()
}
