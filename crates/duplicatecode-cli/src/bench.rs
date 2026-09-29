//! Benchmark: do independent implementations of the same task find each other?
//!
//! Ground truth is the task number. For every query unit in set A, all units of set B (same
//! language family) are ranked by a scorer; the ranking is correct if the top hit belongs to the
//! same task.

use anyhow::{Context, Result};
use duplicatecode_engine::index::{score, score_with, Scores, Weights, FEATURE_NAMES, N_FEATURES};
use duplicatecode_engine::{load_file_units, load_units, Corpus, MatchOptions, Unit};
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

fn load_combos(dataset: &Path, file_level: bool, keep_boilerplate: bool) -> Result<Combos> {
    let targets = strict_targets(dataset)?;
    let mut combos = Combos::new();
    for (set, dir) in [("strict", "impls"), ("loose", "impls-loose"), ("hard", "impls-hard")] {
        let Ok(rd) = std::fs::read_dir(dataset.join(dir)) else { continue };
        let mut models: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.is_dir()).collect();
        models.sort();
        for mdir in models {
            let model = mdir.file_name().unwrap().to_string_lossy().to_string();
            let mut items = Vec::new();
            for u in if file_level { load_file_units(&mdir) } else { load_units(&mdir) } {
                let group = if set == "strict" {
                    let base = u.file.rsplit('/').next().unwrap_or(&u.file);
                    targets.get(base).cloned()
                } else {
                    Some(u.file[..2].to_string())
                };
                if !keep_boilerplate && u.boilerplate {
                    continue;
                }
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

pub fn run(dataset: &Path, min_tokens: usize, negatives: Option<&Path>, file_level: bool, keep_boilerplate: bool, mutations: bool) -> Result<()> {
    let combos = load_combos(dataset, file_level, keep_boilerplate)?;
    println!("level: {}", if file_level { "whole files" } else { "functions/classes" });
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
    if mutations {
        mutation_report(dataset, &combos, &categories, min_tokens)?;
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
    let opts = MatchOptions { threshold: 0.0, min_tokens, top_n: 1, min_lines: 0, min_name: 0.0, weights: Weights::default(), skip_tests: false, skip_boilerplate: true };
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
    let bw = Weights::default();
    let base = |s: &Sample| s.x.iter().zip(bw.w).map(|(x, w)| x * w).sum::<f64>();

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

/// Robustness to mechanical rewrites: how well does a unit still match its own mutated copy?
/// The threshold is the score above which only 1% of different-task pairs fall.
fn mutation_report(
    dataset: &Path,
    combos: &Combos,
    categories: &[(&str, fn(&str, &str) -> bool)],
    min_tokens: usize,
) -> Result<()> {
    use duplicatecode_engine::mutate::{apply, Mutation};
    use duplicatecode_engine::{extract_units, Lang};

    let samples = collect_samples(combos, categories, min_tokens);
    let bw = Weights::default();
    let mut neg: Vec<f64> = samples
        .iter()
        .filter(|s| !s.y)
        .map(|s| s.x.iter().zip(bw.w).map(|(x, w)| x * w).sum::<f64>())
        .collect();
    neg.sort_by(|a, b| a.total_cmp(b));
    let thr = neg[((0.99 * neg.len() as f64) as usize).min(neg.len() - 1)];

    let mut files = Vec::new();
    for dir in ["impls", "impls-loose", "impls-hard"] {
        for e in walkdir::WalkDir::new(dataset.join(dir)).into_iter().filter_map(Result::ok) {
            if e.file_type().is_file() {
                if let Some(lang) = Lang::from_path(e.path()) {
                    files.push((lang, std::fs::read_to_string(e.path())?, e.path().display().to_string()));
                }
            }
        }
    }
    println!("\n== mutation robustness: original vs mutated copy (threshold {thr:.2} = 1% FPR on different-task pairs) ==");
    println!("{:<18}{:>6}{:>10}{:>9}{:>9}{:>10}{:>10}{:>7}{:>12}", "mutation", "n", "recall", "@0.5", "@0.7", "combined", "structure", "name", "stmt_exact");
    for m in Mutation::ALL {
        let (mut n, mut hit, mut h5, mut h7) = (0usize, 0usize, 0usize, 0usize);
        let mut sums = [0.0f64; 4];
        for (lang, src, path) in &files {
            let orig = extract_units(path, *lang, src);
            let mutated = extract_units(path, *lang, &apply(m, *lang, src));
            if orig.len() != mutated.len() {
                continue;
            }
            for (o, t) in orig.iter().zip(&mutated) {
                if o.token_count() < min_tokens || o.boilerplate {
                    continue;
                }
                let sc = score(o, t);
                n += 1;
                hit += (sc.combined > thr) as usize;
                h5 += (sc.combined >= 0.5) as usize;
                h7 += (sc.combined >= 0.7) as usize;
                sums[0] += sc.combined;
                sums[1] += sc.structural;
                sums[2] += sc.name;
                sums[3] += sc.stmt_exact;
            }
        }
        let d = n.max(1) as f64;
        println!(
            "{:<18}{:>6}{:>9.0}%{:>8.0}%{:>8.0}%{:>10.2}{:>10.2}{:>7.2}{:>12.2}",
            m.name(), n, 100.0 * hit as f64 / d, 100.0 * h5 as f64 / d, 100.0 * h7 as f64 / d, sums[0] / d, sums[1] / d, sums[2] / d, sums[3] / d
        );
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct Case {
    id: u32,
    repo_root: String,
    file: String,
    start_line: u32,
    end_line: u32,
}

/// Does a hand-written model re-implementation of a real repo function get matched to the original?
/// Also counts alarms on *other* code (what a maintainer would see as noise).
pub fn reimpl_eval(cases_path: &Path, impls: &Path) -> Result<()> {
    use duplicatecode_engine::{extract_units, Lang};
    use std::collections::BTreeMap;

    let cases: Vec<Case> = serde_json::from_str(&std::fs::read_to_string(cases_path)?)?;
    let mut corpora: BTreeMap<String, Corpus> = BTreeMap::new();
    for c in &cases {
        corpora.entry(c.repo_root.clone()).or_insert_with(|| Corpus::new(load_units(Path::new(&c.repo_root))));
    }
    let mut models: Vec<_> = std::fs::read_dir(impls)?.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.is_dir()).collect();
    models.sort();

    struct Row {
        model: String,
        // per setting: (original's score if it would be reported, best other score)
        per_setting: Vec<(Option<f64>, f64)>,
    }
    let settings: [(&str, Weights, f64); 3] = [
        ("copies, min-name 0.3", Weights::copies(), 0.3),
        ("copies, min-name 0", Weights::copies(), 0.0),
        ("reimpl, min-name 0", Weights::default(), 0.0),
    ];
    let mut rows: Vec<Row> = Vec::new();
    let mut missing = 0;
    for mdir in &models {
        let model = mdir.file_name().unwrap().to_string_lossy().to_string();
        for c in &cases {
            let Some(path) = ["py", "ts", "tsx"].iter().map(|e| mdir.join(format!("{}.{e}", c.id))).find(|p| p.exists()) else {
                missing += 1;
                continue;
            };
            let Some(lang) = Lang::from_path(&path) else { continue };
            let src = std::fs::read_to_string(&path)?;
            let mut units = extract_units("new", lang, &src);
            units.sort_by_key(|u| std::cmp::Reverse(u.token_count()));
            let Some(q) = units.first() else { continue };
            let corpus = &corpora[&c.repo_root];
            let is_orig = |u: &Unit| u.file == c.file && u.start_line <= c.end_line && c.start_line <= u.end_line;
            let mut per_setting = Vec::new();
            for (_, w, min_name) in &settings {
                let opts = MatchOptions { threshold: 0.0, min_tokens: 20, top_n: 8, min_lines: 0, min_name: *min_name, weights: *w, skip_tests: false, skip_boilerplate: false };
                let hits = corpus.best_for(q, opts);
                let orig = hits.iter().find(|(u, _)| is_orig(u)).map(|(_, s)| s.combined);
                let other = hits.iter().find(|(u, _)| !is_orig(u)).map(|(_, s)| s.combined).unwrap_or(0.0);
                per_setting.push((orig, other));
            }
            let _ = score_with;
            rows.push(Row { model: model.clone(), per_setting });
        }
    }
    println!("{} (case, model) re-implementations evaluated ({missing} missing)", rows.len());
    for (k, (label, _, _)) in settings.iter().enumerate() {
        println!("\n== {label} ==");
        let ths = [0.3, 0.4, 0.5, 0.6, 0.7];
        print!("{:<10}{:>5}  original found (score >= t):", "model", "n");
        for t in ths {
            print!("{t:>6.1}");
        }
        print!("   | other-code alarm (score >= t):");
        for t in ths {
            print!("{t:>6.1}");
        }
        println!();
        let mut names: Vec<&str> = rows.iter().map(|r| r.model.as_str()).collect();
        names.dedup();
        let mut groups: Vec<(&str, Vec<&Row>)> = names.iter().map(|m| (*m, rows.iter().filter(|r| r.model == *m).collect())).collect();
        groups.push(("all", rows.iter().collect()));
        for (m, rs) in groups {
            let n = rs.len().max(1) as f64;
            print!("{m:<10}{:>5}  {:<30}", rs.len(), "");
            for t in ths {
                print!("{:>5.0}%", 100.0 * rs.iter().filter(|r| r.per_setting[k].0.is_some_and(|s| s >= t)).count() as f64 / n);
            }
            print!("   | {:<30}", "");
            for t in ths {
                print!("{:>5.0}%", 100.0 * rs.iter().filter(|r| r.per_setting[k].1 >= t).count() as f64 / n);
            }
            println!();
        }
    }
    Ok(())
}
