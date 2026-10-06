//! Labeled-group benchmark: files inside one group are clones, files of different groups are not.
//!
//! Layout `<root>/<dataset>/<group>/<file>`. Every file is one whole-file unit; all pairs inside a
//! dataset are scored. Reports AUC, recall at fixed false-positive rates and top-1 retrieval.

use anyhow::{bail, Result};
use duplicatecode_engine::index::{score_with_idf, Idf, Scores, Weights};
use duplicatecode_engine::{load_file_units, load_units, Unit};
use rayon::prelude::*;
use std::path::Path;

type Pick = fn(&Scores) -> f64;
const SIGNALS: [(&str, Pick); 7] = [
    ("structure", |s| s.structural),
    ("loose", |s| s.loose),
    ("kinds", |s| s.kinds),
    ("stmt_exact", |s| s.stmt_exact),
    ("stmt_shape", |s| s.stmt_shape),
    ("stmt_lcs", |s| s.stmt_lcs),
    ("combined", |s| s.combined),
];

/// Area under the ROC curve with tie handling (Mann-Whitney U).
fn auc(scores: &[(f64, bool)]) -> f64 {
    let mut v: Vec<(f64, bool)> = scores.to_vec();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (mut rank_sum, mut n_pos, mut i) = (0.0, 0usize, 0usize);
    while i < v.len() {
        let mut j = i;
        while j < v.len() && v[j].0 == v[i].0 {
            j += 1;
        }
        let avg_rank = (i + j + 1) as f64 / 2.0; // 1-based average rank of the tie block
        let pos = v[i..j].iter().filter(|x| x.1).count();
        rank_sum += avg_rank * pos as f64;
        n_pos += pos;
        i = j;
    }
    let n_neg = v.len() - n_pos;
    if n_pos == 0 || n_neg == 0 {
        return f64::NAN;
    }
    (rank_sum - (n_pos * (n_pos + 1)) as f64 / 2.0) / (n_pos * n_neg) as f64
}

/// Fraction of positives scoring strictly above the (1 - fpr) quantile of the negatives.
fn tpr_at_fpr(scores: &[(f64, bool)], fpr: f64) -> f64 {
    let mut neg: Vec<f64> = scores.iter().filter(|s| !s.1).map(|s| s.0).collect();
    let pos: Vec<f64> = scores.iter().filter(|s| s.1).map(|s| s.0).collect();
    if neg.is_empty() || pos.is_empty() {
        return f64::NAN;
    }
    neg.sort_by(|a, b| a.total_cmp(b));
    let idx = (((1.0 - fpr) * neg.len() as f64).ceil() as usize).clamp(1, neg.len()) - 1;
    let thr = neg[idx];
    pos.iter().filter(|p| **p > thr).count() as f64 / pos.len() as f64
}

fn group_of(u: &Unit) -> &str {
    u.file.split('/').next().unwrap_or("")
}

struct Report {
    auc: f64,
    tpr1: f64,
    tpr5: f64,
    top1: f64,
}

fn evaluate(units: &[Unit], pick: Pick, all: &[Vec<Scores>]) -> Report {
    let n = units.len();
    let mut pairs = Vec::with_capacity(n * n / 2);
    let mut hit = 0usize;
    for i in 0..n {
        let mut best: Option<(f64, bool)> = None;
        for j in 0..n {
            if i == j || units[i].file == units[j].file {
                continue;
            }
            let v = pick(&all[i][j]);
            let same = group_of(&units[i]) == group_of(&units[j]);
            if j > i {
                pairs.push((v, same));
            }
            if best.is_none_or(|b| v > b.0) {
                best = Some((v, same));
            }
        }
        hit += best.is_some_and(|b| b.1) as usize;
    }
    Report {
        auc: auc(&pairs),
        tpr1: tpr_at_fpr(&pairs, 0.01),
        tpr5: tpr_at_fpr(&pairs, 0.05),
        top1: hit as f64 / n as f64,
    }
}

pub fn run(root: &Path, quiet: bool, dump: Option<&Path>) -> Result<()> {
    let mut dump_out = dump.map(|p| std::fs::File::create(p)).transpose()?;
    let mut datasets: Vec<_> = std::fs::read_dir(root)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir() && !p.file_name().unwrap().to_string_lossy().starts_with('_'))
        .collect();
    datasets.sort();
    if datasets.is_empty() {
        bail!(
            "no datasets below {} (run eval/fetch_codenet.py)",
            root.display()
        );
    }
    let mut headline = Vec::new();
    for dir in datasets {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        // `fn-*` datasets compare functions/classes (product path); the others whole files
        let functions = name.starts_with("fn-");
        let mut units = if functions {
            load_units(&dir)
                .into_iter()
                .filter(|u| !u.boilerplate && u.token_count() >= 8)
                .collect()
        } else {
            load_file_units(&dir)
        };
        units.sort_by(|a, b| (&a.file, a.start_line).cmp(&(&b.file, b.start_line)));
        let n = units.len();
        let idf = Idf::from_units(&units);
        let weights = Weights::default();
        // full score matrix once; each signal is a projection of it
        let all: Vec<Vec<Scores>> = (0..n)
            .into_par_iter()
            .map(|i| {
                (0..n)
                    .map(|j| score_with_idf(&units[i], &units[j], &weights, Some(&idf)))
                    .collect()
            })
            .collect();
        if let Some(f) = dump_out.as_mut() {
            use std::io::Write;
            for i in 0..n {
                for j in i + 1..n {
                    if units[i].file == units[j].file {
                        continue;
                    }
                    let mut feats: Vec<String> = all[i][j]
                        .features()
                        .iter()
                        .map(|x| format!("{x:.5}"))
                        .collect();
                    feats.push(format!("{:.5}", all[i][j].containment));
                    writeln!(
                        f,
                        "{name}\t{}\t{}\t{}\t{}\t{}",
                        group_of(&units[i]),
                        group_of(&units[j]),
                        units[i].tokens.len().min(units[j].tokens.len()),
                        units[i].tokens.len().max(units[j].tokens.len()),
                        feats.join("\t")
                    )?;
                }
            }
        }
        let groups: std::collections::BTreeSet<&str> = units.iter().map(group_of).collect();
        if !quiet {
            println!("== {name}: {n} files, {} groups", groups.len());
            println!(
                "{:<12} {:>6} {:>8} {:>8} {:>7}",
                "signal", "AUC", "TPR@1%", "TPR@5%", "top1"
            );
        }
        for (label, pick) in SIGNALS {
            let r = evaluate(&units, pick, &all);
            if !quiet {
                println!(
                    "{label:<12} {:>6.3} {:>8.3} {:>8.3} {:>7.3}",
                    r.auc, r.tpr1, r.tpr5, r.top1
                );
            }
            if label == "combined" {
                headline.push((name.clone(), (r.auc + r.tpr1) / 2.0));
            }
        }
    }
    for (name, s) in &headline {
        println!("dataset_score {name}={s:.4}");
    }
    let mean = headline.iter().map(|h| h.1).sum::<f64>() / headline.len() as f64;
    println!("score={mean:.4}");
    Ok(())
}

/// Build `<out>/<id>/<variant>.tsx` groups from real components: the original plus mechanically
/// rewritten variants (renames, statement swaps, temp variables, logging, dead code, all combined).
/// Other components act as negatives. `skip` lets a holdout set use disjoint files.
pub fn make_mutation_groups(
    src: &Path,
    out: &Path,
    n: usize,
    seed: u64,
    skip: usize,
) -> Result<()> {
    use duplicatecode_engine::mutate::{apply, Mutation};
    use duplicatecode_engine::{walk_files, Lang};
    let mut files: Vec<_> = walk_files(src, &[])
        .into_iter()
        .filter(|p| {
            let s = p.to_string_lossy();
            s.ends_with(".tsx")
                && !s.contains("node_modules")
                && !s.contains("/.worktrees/")
                && !s.contains("/.claude/")
                && !s.contains(".test.")
                && !s.contains(".stories.")
                && !s.contains("/generated/")
        })
        .filter(|p| {
            std::fs::read_to_string(p)
                .map(|t| (30..=300).contains(&t.lines().count()) && t.contains("</"))
                .unwrap_or(false)
        })
        .collect();
    files.sort();
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for i in (1..files.len()).rev() {
        files.swap(i, (next() % (i as u64 + 1)) as usize);
    }
    let mut written = 0;
    for (k, p) in files.iter().skip(skip).take(n).enumerate() {
        let text = std::fs::read_to_string(p)?;
        let dir = out.join(format!("c{:03}", skip + k));
        std::fs::create_dir_all(&dir)?;
        std::fs::write(dir.join("orig.tsx"), &text)?;
        for m in Mutation::ALL {
            if m == Mutation::LoopToComprehension {
                continue; // Python only
            }
            let v = apply(m, Lang::Tsx, &text);
            if v != text {
                std::fs::write(dir.join(format!("{}.tsx", m.name().replace(' ', "_"))), v)?;
            }
        }
        written += 1;
    }
    println!("wrote {written} component groups to {}", out.display());
    Ok(())
}
