# /// script
# requires-python = ">=3.10,<3.13"
# dependencies = ["embed-anything", "numpy", "pandas", "scikit-learn"]
# ///
"""Embedder bake-off with embed-anything on clean (uncontaminated) clone data.

Embeds every function/class unit of the fn-* datasets and every SQL file, for the dev and holdout roots,
then scores each model by (a) cosine alone and (b) a fixed blend with the static score, using the
per-pair feature dumps written by `duplicatecode eval-groups --dump`.

    uv run --no-project --python 3.12 eval/embed_bench.py <model-id> [<model-id> ...] \
        --dev eval/data/codenet --hold eval/data/holdout --dump-dev dev.tsv --dump-hold hold.tsv --out bench.json
"""
import argparse, json, pathlib, re, subprocess, time

import numpy as np, pandas as pd
import embed_anything
from embed_anything import EmbeddingModel
from sklearn.metrics import roc_auc_score

BIN = "duplicatecode"
F = ["structural", "loose", "kinds", "literals", "api", "name", "callees", "stmt_exact", "stmt_shape",
     "stmt_lcs", "containment", "embed"]
W_FN = np.array([0.16, 0.03, 0.02, 0.13, 0.23, 0.17, 0, 0, 0, 0, 0.26, 0])
W_SQL = np.array([0, 0.46, 0, 0.17, 0.29, 0.02, 0.06, 0, 0, 0, 0, 0])


def collect(root: pathlib.Path, tag: str):
    """(key, text) for fn-* units and sql files; keys match the eval-groups dump refs."""
    items = []
    for ds in sorted(p for p in root.glob("fn-*") if p.is_dir()):
        for g in sorted(p for p in ds.iterdir() if p.is_dir()):
            out = subprocess.run([BIN, "units", str(g)], capture_output=True, text=True).stdout
            for line in out.splitlines():
                m = re.match(r"(.+?):(\d+)-(\d+) (\w+) (\S+) \((\d+) tokens\)", line)
                if not m or int(m.group(6)) < 8:
                    continue
                f, a, b = m.group(1), int(m.group(2)), int(m.group(3))
                text = "\n".join((g / f).read_text(errors="ignore").splitlines()[a - 1:b])
                items.append((f"{tag}|{ds.name}|{g.name}/{f}:{a}", text[:2500]))
    sql = root / "sql"
    if sql.is_dir():
        for g in sorted(p for p in sql.iterdir() if p.is_dir()):
            for f in sorted(g.glob("*.sql")):
                items.append((f"{tag}|sql|{g.name}/{f.name}:1", f.read_text(errors="ignore")[:2500]))
    return items


def metrics(y, s):
    thr = np.quantile(s[y == 0], 0.99)
    auc, tpr = roc_auc_score(y, s), float((s[y == 1] > thr).mean())
    return (auc + tpr) / 2


def evaluate(vecs: dict, dumps: dict) -> dict:
    res = {}
    for tag, d in dumps.items():
        for ds in ("fn-python", "fn-typescript", "sql"):
            b = d[d.ds == ds].copy()
            ka = [f"{tag}|{ds}|{x}" for x in b.fa]
            kb = [f"{tag}|{ds}|{x}" for x in b.fb]
            ok = np.array([x in vecs and y in vecs for x, y in zip(ka, kb)])
            b, ka, kb = b[ok], np.array(ka)[ok], np.array(kb)[ok]
            cos = np.array([float(vecs[x] @ vecs[y]) for x, y in zip(ka, kb)])
            y = (b.ga == b.gb).astype(int).values
            w = W_SQL if ds == "sql" else W_FN
            static = b[F].values @ w
            res[f"{tag}:{ds}"] = {
                "static": metrics(y, static),
                "cosine": metrics(y, np.clip(cos, 0, 1)),
                "blend35": metrics(y, 0.65 * static + 0.35 * np.clip(cos, 0, 1)),
            }
    return res


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("models", nargs="+")
    ap.add_argument("--dev", default="eval/data/codenet")
    ap.add_argument("--hold", default="eval/data/holdout")
    ap.add_argument("--dump-dev", required=True)
    ap.add_argument("--dump-hold", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--bin", default="duplicatecode")
    a = ap.parse_args()
    BIN = a.bin
    cols = ["ds", "fa", "fb", "ga", "gb", "lmin", "lmax"] + F
    dumps = {"dev": pd.read_csv(a.dump_dev, sep="\t", header=None, names=cols),
             "hold": pd.read_csv(a.dump_hold, sep="\t", header=None, names=cols)}
    items = collect(pathlib.Path(a.dev), "dev") + collect(pathlib.Path(a.hold), "hold")
    keys, texts = [k for k, _ in items], [t for _, t in items]
    print(f"{len(texts)} texts, mean {np.mean([len(t) for t in texts]):.0f} chars", flush=True)
    out = json.loads(pathlib.Path(a.out).read_text()) if pathlib.Path(a.out).exists() else {}
    for mid in a.models:
        if mid in out:
            continue
        try:
            t0 = time.time()
            model = EmbeddingModel.from_pretrained_hf(mid)
            load = time.time() - t0
            t0 = time.time()
            vecs = []
            for i in range(0, len(texts), 16):
                vecs += [r.embedding for r in embed_anything.embed_query(texts[i:i + 16], model)]
            secs = time.time() - t0
            v = np.array(vecs, dtype=np.float32)
            v /= np.linalg.norm(v, axis=1, keepdims=True) + 1e-9
            res = evaluate(dict(zip(keys, v)), dumps)
            out[mid] = {"dim": int(v.shape[1]), "sec_per_text": secs / len(texts), "load_s": load, "scores": res}
            print(f"{mid}: dim={v.shape[1]} {secs/len(texts)*1000:.0f} ms/text", flush=True)
        except BaseException as e:  # panics from the native loader are BaseException
            out[mid] = {"error": str(e)[:200]}
            print(f"{mid}: FAILED {str(e)[:120]}", flush=True)
        pathlib.Path(a.out).write_text(json.dumps(out, indent=1))
