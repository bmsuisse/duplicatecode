//! Benchmark: do independent implementations of the same task find each other?
//!
//! Ground truth is the task number. For every query unit in set A, all units of set B (same
//! language family) are ranked by a scorer; the ranking is correct if the top hit belongs to the
//! same task.

use anyhow::{Context, Result};
use duplicatecode_engine::index::{score, Scores};
use duplicatecode_engine::{load_units, Unit};
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
    for (set, dir) in [("strict", "impls"), ("loose", "impls-loose")] {
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

pub fn run(dataset: &Path, min_tokens: usize) -> Result<()> {
    let combos = load_combos(dataset)?;
    for (name, items) in &combos {
        println!("loaded {name}: {} units", items.len());
    }

    // (category label, predicate over (query combo, candidate combo))
    let categories: [(&str, fn(&str, &str) -> bool); 4] = [
        ("strict~strict (cross-model)", |a, b| kind(a) == "strict" && kind(b) == "strict" && a != b),
        ("loose~loose  (cross-model)", |a, b| kind(a) == "loose" && kind(b) == "loose" && a != b),
        ("strict~loose (cross-model)", |a, b| kind(a) != kind(b) && model(a) != model(b)),
        ("strict~loose (same model) ", |a, b| kind(a) != kind(b) && model(a) == model(b)),
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
    Ok(())
}

fn kind(combo: &str) -> &str {
    combo.split('/').next().unwrap()
}
fn model(combo: &str) -> &str {
    combo.split('/').nth(1).unwrap()
}
