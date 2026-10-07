# /// script
# requires-python = ">=3.10"
# dependencies = ["numpy==2.5.3"]
# ///
"""Tables for the light-embedding bake-off (issue #12) from the files written by embed_codebases.py.

    uv run --no-project eval/analyze_codebases.py --tasks cb/tasks --results cb/results --max-corpus 800
"""
import argparse, glob, json, pathlib, random

import numpy as np

# parameters in millions (approximate; from the model cards)
PARAMS = {"all-MiniLM-L6-v2": 22.7, "all-MiniLM-L12-v2": 33.4, "paraphrase-MiniLM-L3-v2": 17.4, "multi-qa-MiniLM-L6-cos-v1": 22.7,
          "bge-small-en-v1.5": 33.4, "bge-micro-v2": 17.4, "gte-small": 33.4, "gte-tiny": 22.7, "e5-small-v2": 33.4,
          "snowflake-arctic-embed-xs": 22.6, "snowflake-arctic-embed-s": 33.2, "mxbai-embed-xsmall-v1": 22.7,
          "jina-embeddings-v2-small-en": 33.0, "potion-base-2M": 1.9, "potion-base-4M": 3.7, "potion-base-8M": 7.6,
          "potion-base-32M": 32.3, "potion-retrieval-32M": 32.3, "bge-base-en-v1.5": 109.0}
PUBLIC = ["py-requests", "py-flask", "js-lodash", "js-axios", "ts-datefns"]
INTERNAL = ["internal-py", "internal-sql", "internal-tsx"]
LAMBDA = 0.35  # the shipped --embed-weight; the blend here is (1 - l) * static + l * cosine


def task(d: pathlib.Path, cap: int):
    corpus = [json.loads(l) for l in (d / "corpus.jsonl").read_text().splitlines()]
    impls = [json.loads(l) for l in (d / "impls.jsonl").read_text().splitlines()]
    want = {i["id"] for i in impls}
    keep = list(range(len(corpus)))
    if cap and len(corpus) > cap:  # same selection as embed_codebases.load_task
        rest = [u for u in corpus if u["id"] not in want]
        random.Random(7).shuffle(rest)
        ids = [u["id"] for u in corpus if u["id"] in want] + [u["id"] for u in rest[: cap - len(want)]]
        pos = {u["id"]: i for i, u in enumerate(corpus)}
        keep = [pos[i] for i in ids]
        corpus = [corpus[i] for i in keep]
    idx = {u["id"]: i for i, u in enumerate(corpus)}
    return keep, [idx[i["id"]] for i in impls]


def ranks(sim: np.ndarray, gold: list[int]) -> np.ndarray:
    return np.array([int((sim[q] > sim[q, g]).sum()) + 1 for q, g in enumerate(gold)])


def stats(r: np.ndarray) -> dict:
    return {"r1": float((r <= 1).mean()), "r5": float((r <= 5).mean()), "mrr": float((1 / r).mean())}


def mean(vals):
    return float(np.mean(vals)) if len(vals) else float("nan")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--tasks", type=pathlib.Path, required=True)
    ap.add_argument("--results", type=pathlib.Path, required=True)
    ap.add_argument("--max-corpus", type=int, default=800)
    a = ap.parse_args()
    names = [p.name for p in sorted(a.tasks.iterdir()) if (p / "impls.jsonl").exists()]
    info = {n: task(a.tasks / n, a.max_corpus) for n in names}
    static = {n: np.load(a.results / f"cos-STATIC-{n}.npy")[:, info[n][0]] for n in names}
    srank = {n: ranks(static[n], info[n][1]) for n in names}
    timing = {}
    for f in glob.glob(str(a.results / "timing-*.json")):
        t = json.loads(pathlib.Path(f).read_text())
        timing[t["model"].split("/")[-1]] = t["ms_per_text"]
    models = sorted({pathlib.Path(f).stem.split("-", 1)[1] for f in glob.glob(str(a.results / "quality-*.json")) if "masked" not in f})
    rows = []
    for safe in models:
        short = safe.split("__")[-1]
        row = {"model": short, "params": PARAMS.get(short), "ms": timing.get(short)}
        for variant in ("", "-masked"):
            tag = variant
            per = {}
            for n in names:
                f = a.results / f"cos{tag}-{safe}-{n}.npy"
                if not f.exists():
                    continue
                cos = np.load(f)
                gold = info[n][1]
                r = ranks(cos, gold)
                blend = (1 - LAMBDA) * static[n] + LAMBDA * cos
                rb = ranks(blend, gold)
                miss = srank[n] > 1  # queries the static detector alone gets wrong
                per[n] = {**stats(r), "blend_r1": float((rb <= 1).mean()), "blend_mrr": float((1 / rb).mean()),
                          "fixed_by_embedding": float((r[miss] <= 1).mean()) if miss.any() else float("nan"),
                          "fixed_by_blend": float((rb[miss] <= 1).mean()) if miss.any() else float("nan")}
            if not per:
                continue
            for grp, g in (("all", names), ("public", PUBLIC), ("internal", INTERNAL)):
                sel = [per[n] for n in g if n in per]
                row[f"{grp}{tag}_r1"] = mean([s["r1"] for s in sel])
                row[f"{grp}{tag}_mrr"] = mean([s["mrr"] for s in sel])
                row[f"{grp}{tag}_blend_r1"] = mean([s["blend_r1"] for s in sel])
                row[f"{grp}{tag}_fixed"] = mean([s["fixed_by_embedding"] for s in sel])
                row[f"{grp}{tag}_fixed_blend"] = mean([s["fixed_by_blend"] for s in sel])
            row[f"per{tag}"] = per
        rows.append(row)
    base = {g: {"r1": mean([stats(srank[n])["r1"] for n in grp]), "mrr": mean([stats(srank[n])["mrr"] for n in grp])}
            for g, grp in (("all", names), ("public", PUBLIC), ("internal", INTERNAL))}
    print("static detector alone (no embeddings):", {g: {k: round(v, 3) for k, v in d.items()} for g, d in base.items()})
    fmt = lambda x, d=3: "n/a" if x is None or x != x else f"{x:.{d}f}"
    print("\n| model | M params | ms/text | R@1 all | R@1 public | R@1 internal | MRR all | R@1 name-masked (all) | blend R@1 (all) | static misses fixed by embedding / blend |")
    print("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |")
    for r in sorted(rows, key=lambda r: -r.get("all_mrr", 0)):
        print(f"| {r['model']} | {fmt(r['params'], 1)} | {fmt(r['ms'], 1)} | {fmt(r.get('all_r1'))} | {fmt(r.get('public_r1'))} | "
              f"{fmt(r.get('internal_r1'))} | {fmt(r.get('all_mrr'))} | {fmt(r.get('all-masked_r1'))} | {fmt(r.get('all_blend_r1'))} | "
              f"{fmt(r.get('all_fixed'), 2)} / {fmt(r.get('all_fixed_blend'), 2)} |")
    (a.results / "summary.json").write_text(json.dumps({"static": base, "rows": rows}, indent=1))


if __name__ == "__main__":
    main()
