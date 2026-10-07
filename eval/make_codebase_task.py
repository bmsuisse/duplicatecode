"""Build one codebase's retrieval task: a corpus of all its functions and a sample of target functions.

The targets are later described in words (agent A) and re-implemented from that description alone
(agent B); a model is then scored on how high it ranks the original among the whole corpus.

    python3 eval/make_codebase_task.py --name py-requests --root repos/requests/src --ext py \
        --out cb/py-requests --n 30 [--exclude test --exclude tests] [--max-corpus 1500] [--seed 1]

Output: corpus.jsonl (id, file, name, kind, start, end, text), targets.jsonl (a sample of the corpus
with text) and a README line. Private repositories stay local: only aggregate numbers are published.
"""
import argparse, json, pathlib, random, re, subprocess, sys

UNIT = re.compile(r"^(.+?):(\d+)-(\d+) (\w+) (\S+) \((\d+) tokens\)$")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--name", required=True)
    ap.add_argument("--root", required=True, type=pathlib.Path)
    ap.add_argument("--ext", required=True, help="comma separated file extensions without dot")
    ap.add_argument("--out", required=True, type=pathlib.Path)
    ap.add_argument("--n", type=int, default=30)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--max-corpus", type=int, default=1500)
    ap.add_argument("--per-file", type=int, default=2, help="at most this many targets from one file")
    ap.add_argument("--min-lines", type=int, default=8, help="smallest target, in lines")
    ap.add_argument("--min-tokens", type=int, default=40, help="smallest target, in normalized tokens")
    ap.add_argument("--exclude", action="append", default=[], help="path substring to skip (relative path)")
    ap.add_argument("--bin", default="duplicatecode")
    a = ap.parse_args()
    exts = tuple("." + e for e in a.ext.split(","))
    skip = ["node_modules", "/dist/", "/build/", "__pycache__", ".d.ts", "/generated/", ".min.", *a.exclude]
    out = subprocess.run([a.bin, "units", str(a.root)], capture_output=True, text=True, check=True).stdout
    corpus = []
    for line in out.splitlines():
        m = UNIT.match(line)
        if not m:
            continue
        rel, s, e, kind, name, tokens = m.group(1), int(m.group(2)), int(m.group(3)), m.group(4), m.group(5), int(m.group(6))
        if not rel.endswith(exts) or any(x in "/" + rel for x in skip) or kind == "class":
            continue
        if name.startswith("__") or tokens < 25 or e - s + 1 > 120:
            continue
        text = "\n".join((a.root / rel).read_text(errors="ignore").splitlines()[s - 1:e])
        corpus.append({"file": rel, "name": name, "kind": kind, "start": s, "end": e, "tokens": tokens, "text": text[:3000]})
    if len(corpus) < a.n * 2:
        sys.exit(f"{a.name}: only {len(corpus)} usable units")
    rnd = random.Random(a.seed)
    # targets: medium sized, distinct names, spread over files
    pool = [u for u in corpus if a.min_lines <= u["end"] - u["start"] + 1 <= 50 and u["tokens"] >= a.min_tokens]
    rnd.shuffle(pool)
    targets, names = [], set()
    per_file: dict[str, int] = {}
    for u in pool:
        if u["name"] in names or per_file.get(u["file"], 0) >= a.per_file:
            continue
        names.add(u["name"])
        per_file[u["file"]] = per_file.get(u["file"], 0) + 1
        targets.append(u)
        if len(targets) == a.n:
            break
    if len(targets) < a.n:
        sys.exit(f"{a.name}: only {len(targets)} targets")
    ids = {id(t) for t in targets}
    rest = [u for u in corpus if id(u) not in ids]
    rnd.shuffle(rest)
    kept = targets + rest[: max(0, a.max_corpus - len(targets))]
    rnd.shuffle(kept)
    a.out.mkdir(parents=True, exist_ok=True)
    target_ids = set()
    with (a.out / "corpus.jsonl").open("w") as f:
        for i, u in enumerate(kept):
            u["id"] = f"{a.name}:{i}"
            if id(u) in ids:
                target_ids.add(u["id"])
            f.write(json.dumps(u) + "\n")
    with (a.out / "targets.jsonl").open("w") as f:
        for u in kept:
            if u["id"] in target_ids:
                f.write(json.dumps({"id": u["id"], "text": u["text"]}) + "\n")
    print(f"{a.name}: corpus {len(kept)} of {len(corpus)} units, {len(targets)} targets -> {a.out}")


if __name__ == "__main__":
    main()
