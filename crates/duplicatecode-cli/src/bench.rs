//! Benchmark: do independent implementations of the same task find each other?
//!
//! Ground truth is the task number. For every query unit in set A, all units of set B (same
//! language family) are ranked by a scorer; the ranking is correct if the top hit belongs to the
//! same task.

use anyhow::{Context, Result};
use duplicatecode_engine::index::{score, Scores, Weights, FEATURE_NAMES, N_FEATURES};
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
    pair_report(&combos, &categories, min_tokens);
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

struct Sample {
    cat: usize,
    fam: usize,
    task: u32,
    x: [f64; N_FEATURES],
    y: bool,
}

const FAMILIES: [&str; 2] = ["python", "typescript"];

fn collect_samples(combos: &Combos, categories: &[(&str, fn(&str, &str) -> bool)], min_tokens: usize) -> Vec<Sample> {
    let mut out = Vec::new();
    for (cat, (_, pred)) in categories.iter().enumerate() {
        for (an, aitems) in combos {
            for (bn, bitems) in combos {
                if !pred(an, bn) {
                    continue;
                }
                for q in aitems.iter().filter(|i| i.unit.token_count() >= min_tokens) {
                    let fam = FAMILIES.iter().position(|f| *f == q.unit.lang.family()).unwrap();
                    for c in bitems.iter().filter(|c| {
                        c.unit.lang.family() == q.unit.lang.family() && c.unit.token_count() >= min_tokens
                    }) {
                        out.push(Sample {
                            cat,
                            fam,
                            task: q.group.parse().unwrap_or(0),
                            x: score(&q.unit, &c.unit).features(),
                            y: q.group == c.group,
                        });
                    }
                }
            }
        }
    }
    out
}

fn sigmoid(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}

/// Class-balanced, L2-regularised logistic regression by batch gradient descent.
fn fit(samples: &[&Sample]) -> Weights {
    let pos = samples.iter().filter(|s| s.y).count().max(1) as f64;
    let neg = samples.iter().filter(|s| !s.y).count().max(1) as f64;
    let n = samples.len() as f64;
    let (wp, wn) = (n / (2.0 * pos), n / (2.0 * neg));
    let mut w = Weights { bias: 0.0, w: [0.0; N_FEATURES] };
    for _ in 0..2500 {
        let mut gb = 0.0;
        let mut gw = [0.0; N_FEATURES];
        for s in samples {
            let z = w.bias + s.x.iter().zip(w.w).map(|(x, w)| x * w).sum::<f64>();
            let err = (sigmoid(z) - if s.y { 1.0 } else { 0.0 }) * if s.y { wp } else { wn };
            gb += err;
            for k in 0..N_FEATURES {
                gw[k] += err * s.x[k];
            }
        }
        w.bias -= 1.0 * gb / n;
        for k in 0..N_FEATURES {
            w.w[k] -= 1.0 * (gw[k] / n + 0.001 * w.w[k]);
        }
    }
    w
}

fn apply(w: &Weights, x: &[f64; N_FEATURES]) -> f64 {
    sigmoid(w.bias + x.iter().zip(w.w).map(|(x, w)| x * w).sum::<f64>())
}

/// Fraction of positives scoring above the (1 - fpr) quantile of the negatives.
fn tpr_at_fpr(scores: &[(f64, bool)], fpr: f64) -> f64 {
    let mut neg: Vec<f64> = scores.iter().filter(|s| !s.1).map(|s| s.0).collect();
    let pos: Vec<f64> = scores.iter().filter(|s| s.1).map(|s| s.0).collect();
    if neg.is_empty() || pos.is_empty() {
        return f64::NAN;
    }
    neg.sort_by(|a, b| a.total_cmp(b));
    let idx = (((1.0 - fpr) * neg.len() as f64).ceil() as usize).min(neg.len()) - 1;
    let thr = neg[idx];
    pos.iter().filter(|p| **p > thr).count() as f64 / pos.len() as f64
}

/// Recall of same-task pairs at a fixed false-positive rate among different-task pairs.
fn pair_report(combos: &Combos, categories: &[(&str, fn(&str, &str) -> bool)], min_tokens: usize) {
    let samples = collect_samples(combos, categories, min_tokens);
    let refs: Vec<&Sample> = samples.iter().collect();
    // 2-fold cross-validation over task parity so the fitted score is never scored on tasks it saw.
    let fit_even = fit(&refs.iter().copied().filter(|s| s.task % 2 == 0).collect::<Vec<_>>());
    let fit_odd = fit(&refs.iter().copied().filter(|s| s.task % 2 == 1).collect::<Vec<_>>());
    let cv = |s: &Sample| apply(if s.task % 2 == 0 { &fit_odd } else { &fit_even }, &s.x);
    let base = |s: &Sample| 0.6 * s.x[0] + 0.2 * s.x[5] + 0.2 * s.x[6];

    for (fam_idx, fam) in FAMILIES.iter().enumerate() {
        println!("\n== {fam}: recall of same-task pairs at 1% false-positive rate (different-task pairs) ==");
        print!("{:<30}{:>9}{:>9}", "", "baseline", "fit(cv)");
        for n in FEATURE_NAMES {
            print!("{n:>11}");
        }
        println!();
        for (cat, (label, _)) in categories.iter().enumerate() {
            let sel: Vec<&Sample> = refs.iter().copied().filter(|s| s.cat == cat && s.fam == fam_idx).collect();
            let col = |f: &dyn Fn(&Sample) -> f64| {
                let v: Vec<(f64, bool)> = sel.iter().map(|s| (f(s), s.y)).collect();
                tpr_at_fpr(&v, 0.01)
            };
            print!("{label:<30}{:>8.0}%{:>8.0}%", 100.0 * col(&base), 100.0 * col(&cv));
            for k in 0..N_FEATURES {
                print!("{:>10.0}%", 100.0 * col(&|s: &Sample| s.x[k]));
            }
            println!();
        }
    }

    let all = fit(&refs);
    println!("\nfitted on all data: bias={:.2}", all.bias);
    for (n, w) in FEATURE_NAMES.iter().zip(all.w) {
        println!("  {n:<11}{w:>8.2}");
    }
    let pooled: Vec<(f64, bool)> = refs.iter().map(|s| (apply(&all, &s.x), s.y)).collect();
    let mut neg: Vec<f64> = pooled.iter().filter(|p| !p.1).map(|p| p.0).collect();
    neg.sort_by(|a, b| a.total_cmp(b));
    for q in [0.99, 0.995, 0.999] {
        println!("  score threshold at {:.1}% pooled FPR: {:.3}", 100.0 * (1.0 - q), neg[(q * neg.len() as f64) as usize - 1]);
    }
}
