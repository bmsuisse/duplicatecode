# /// script
# requires-python = ">=3.10,<3.13"
# dependencies = ["embed-anything==0.7.1", "numpy==2.5.3"]
# ///
"""Light-embedding bake-off across codebases (issue #12).

Task per codebase (see make_codebase_task.py): rank every function of the codebase by cosine similarity
to an independent re-implementation of one of them; the original must come out on top. Reports
recall@1/5/10 and MRR per codebase and the macro average, and the cosine matrices (for blending with
the static score offline). Speed is measured separately in --timing mode, on fixed texts and threads.

    uv run --no-project --python 3.12 eval/embed_codebases.py --tasks cb/tasks --out cb/results \
        --models sentence-transformers/all-MiniLM-L6-v2 BAAI/bge-small-en-v1.5
    uv run --no-project --python 3.12 eval/embed_codebases.py --tasks cb/tasks --out cb/results --timing \
        --models ...            # throughput on 300 fixed texts, run alone on an idle machine
    uv run --no-project --python 3.12 eval/embed_codebases.py --probe --models ...   # only: does it load?
"""
import argparse, json, pathlib, random, re, time

import json as _json
import os
import urllib.request

import numpy as np
import embed_anything

OLLAMA = os.environ.get("OLLAMA_HOST_URL", "http://127.0.0.1:11434")  # `ollama:<model>` ids use this server (GPU)

# some models are trained with a prefix; both sides get the same one (the task is symmetric)
PREFIX = {"intfloat/e5-small-v2": "query: ", "intfloat/e5-base-v2": "query: "}
MAX_CHARS = 3000
BATCH = 32


def ollama_embed(name: str, batch: list[str]) -> list[list[float]]:
    req = urllib.request.Request(f"{OLLAMA}/api/embed", _json.dumps({"model": name, "input": batch, "truncate": True}).encode(),
                                 {"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=600) as r:
        return _json.load(r)["embeddings"]


def embed(model, texts: list[str]) -> np.ndarray:
    out = []
    for i in range(0, len(texts), BATCH):
        if isinstance(model, str):
            out += ollama_embed(model, texts[i:i + BATCH])
        else:
            out += [r.embedding for r in embed_anything.embed_query(texts[i:i + BATCH], model)]
    v = np.array(out, dtype=np.float32)
    v /= np.linalg.norm(v, axis=1, keepdims=True) + 1e-9
    return v


DEF = {"py": r"\bdef\s+(\w+)", "js": r"\bfunction\s+(\w+)|\b(?:const|let|var)\s+(\w+)\s*=",
       "ts": r"\bfunction\s+(\w+)|\b(?:const|let|var)\s+(\w+)\s*=", "tsx": r"\bfunction\s+(\w+)|\b(?:const|let|var)\s+(\w+)\s*="}


def mask(text: str, name: str | None, ext: str) -> str:
    """Replace the function's own name by NAME: an embedder must not win on a shared name alone."""
    if not name and ext in DEF:
        m = re.search(DEF[ext], text)
        name = next((g for g in (m.groups() if m else ()) if g), None)
    if name and len(name) >= 3 and ext != "sql":
        text = re.sub(r"\b" + re.escape(name) + r"\b", "NAME", text)
    return text


def load_task(d: pathlib.Path, max_corpus: int = 0):
    corpus = [json.loads(l) for l in (d / "corpus.jsonl").read_text().splitlines()]
    impls = [json.loads(l) for l in (d / "impls.jsonl").read_text().splitlines()]
    if max_corpus and len(corpus) > max_corpus:  # keep every target, subsample the distractors deterministically
        want = {i["id"] for i in impls}
        rest = [u for u in corpus if u["id"] not in want]
        random.Random(7).shuffle(rest)
        corpus = [u for u in corpus if u["id"] in want] + rest[: max_corpus - len(want)]
    index = {u["id"]: i for i, u in enumerate(corpus)}
    gold = [index[i["id"]] for i in impls]
    return corpus, impls, gold


def metrics(sim: np.ndarray, gold: list[int]) -> dict:
    ranks = []
    for q, g in enumerate(gold):
        ranks.append(int((sim[q] > sim[q, g]).sum()) + 1)  # ties count in favour of the gold: negligible for floats
    r = np.array(ranks)
    return {"r1": float((r <= 1).mean()), "r5": float((r <= 5).mean()), "r10": float((r <= 10).mean()),
            "mrr": float((1.0 / r).mean())}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--tasks", type=pathlib.Path)
    ap.add_argument("--out", type=pathlib.Path)
    ap.add_argument("--models", nargs="+", required=True)
    ap.add_argument("--probe", action="store_true")
    ap.add_argument("--mask-names", action="store_true", help="replace the function's own name by NAME on both sides")
    ap.add_argument("--max-corpus", type=int, default=0, help="cap the corpus (targets are always kept)")
    ap.add_argument("--timing", action="store_true")
    a = ap.parse_args()

    for mid in a.models:
        safe = mid.replace("/", "__").replace(":", "_")
        try:
            t0 = time.time()
            model = mid[7:] if mid.startswith("ollama:") else embed_anything.EmbeddingModel.from_pretrained_hf(mid)
            load_s = time.time() - t0
            pre = PREFIX.get(mid, "")
            if a.probe:
                v = embed(model, [pre + "def f(x):\n    return x + 1"])
                print(json.dumps({"model": mid, "ok": True, "dim": int(v.shape[1]), "load_s": round(load_s, 1)}), flush=True)
                continue
            a.out.mkdir(parents=True, exist_ok=True)
            tasks = sorted(p for p in a.tasks.iterdir() if (p / "impls.jsonl").exists())
            if a.timing:
                texts = []
                for d in tasks:  # 320 fixed texts: 40 corpus functions per codebase (deterministic)
                    corpus, _, _ = load_task(d)
                    texts += [pre + u["text"][:MAX_CHARS] for u in corpus[:40]]
                embed(model, texts[:16])  # warm-up
                runs = []
                for _ in range(3):
                    t0 = time.time()
                    embed(model, texts)
                    runs.append((time.time() - t0) / len(texts) * 1000)
                res = {"model": mid, "ms_per_text": float(np.median(runs)), "runs": runs, "n_texts": len(texts), "load_s": load_s,
                       "dim": int(embed(model, texts[:1]).shape[1])}
                (a.out / f"timing-{safe}.json").write_text(json.dumps(res))
                print(json.dumps({k: (round(v, 2) if isinstance(v, float) else v) for k, v in res.items() if k != "runs"}), flush=True)
                continue
            res = {"model": mid, "load_s": load_s, "codebases": {}, "masked": a.mask_names, "max_corpus": a.max_corpus}
            tag = "-masked" if a.mask_names else ""
            for d in tasks:
                corpus, impls, gold = load_task(d, a.max_corpus)
                ext = corpus[0]["file"].rsplit(".", 1)[-1]
                ctext = [mask(u["text"], u["name"], ext) if a.mask_names else u["text"] for u in corpus]
                qtext = [mask(i["code"], None, ext) if a.mask_names else i["code"] for i in impls]
                t0 = time.time()
                c = embed(model, [pre + t[:MAX_CHARS] for t in ctext])
                corpus_s = time.time() - t0
                q = embed(model, [pre + t[:MAX_CHARS] for t in qtext])
                sim = q @ c.T
                np.save(a.out / f"cos{tag}-{safe}-{d.name}.npy", sim.astype(np.float32))
                res["codebases"][d.name] = {**metrics(sim, gold), "corpus": len(corpus), "queries": len(impls),
                                            "ms_per_text": corpus_s / len(corpus) * 1000}
            for k in ("r1", "r5", "r10", "mrr"):
                res[k] = float(np.mean([v[k] for v in res["codebases"].values()]))
            (a.out / f"quality{tag}-{safe}.json").write_text(json.dumps(res, indent=1))
            print(f"{mid}: macro r1={res['r1']:.3f} r5={res['r5']:.3f} mrr={res['mrr']:.3f}", flush=True)
        except BaseException as e:  # the native loader panics (BaseException) on unsupported models
            if isinstance(e, (KeyboardInterrupt, SystemExit)):
                raise
            print(json.dumps({"model": mid, "ok": False, "error": str(e)[:160]}), flush=True)


if __name__ == "__main__":
    main()
