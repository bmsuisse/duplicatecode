"""Evaluate `duplicatecode find`: loose task descriptions as queries over the strict implementations.

    DUPLICATECODE_EMBED_ENDPOINT=http://127.0.0.1:8100/v1 DUPLICATECODE_EMBED_API_KEY=local \
    DUPLICATECODE_EMBED_MODEL=minilm python3 eval/eval_find.py dataset duplicatecode [distractor-dir ...]
"""
import json, pathlib, re, subprocess, sys

dataset, binary = pathlib.Path(sys.argv[1]), sys.argv[2]
target = {}  # strict file name -> task number
for md in sorted((dataset / "tasks").glob("*.md")):
    m = re.search(r"\*\*Target file:\*\*\s*`([^`]+)`", md.read_text())
    if m:
        target[m.group(1)] = md.name[:2]
corpus = [str(dataset / "impls" / m) for m in ("haiku", "sonnet")] + sys.argv[3:]  # optional distractor dirs
ranks = []
for q in sorted((dataset / "tasks-loose").glob("*.md")):
    task, text = q.name[:2], q.read_text().strip()
    out = subprocess.run([binary, "find", text, *corpus, "--top", "20", "--json", "--min-tokens", "1"],
                         capture_output=True, text=True)
    hits = json.loads(out.stdout) if out.stdout.strip() else []
    rank = next((i + 1 for i, h in enumerate(hits) if target.get(pathlib.Path(h["file"]).name) == task), None)
    ranks.append(rank)
n = len(ranks)
rr = sum(1 / r for r in ranks if r) / n
print(f"queries={n} MRR={rr:.3f} " + " ".join(f"R@{k}={sum(1 for r in ranks if r and r <= k) / n:.3f}" for k in (1, 3, 5, 10)))
