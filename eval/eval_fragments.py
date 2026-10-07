"""Score `duplicatecode fragments` on a make_fragment_data.py directory: recall of injected blocks."""
import json, pathlib, subprocess, sys

d, binary = pathlib.Path(sys.argv[1]), sys.argv[2]
extra = sys.argv[3:]
truth = json.loads((d / "truth.json.txt").read_text())
# constructors/dunders are boilerplate by design and never reported; do not count them as misses
def is_dunder(f): return (d / f).read_text().lstrip().startswith("def __")
truth = [t for t in truth if not (is_dunder(t["donor"]) or is_dunder(t["target"]))]
out = subprocess.run([binary, "fragments", str(d), "--json", *extra], capture_output=True, text=True).stdout
found = json.loads(out) if out.strip() else []
hit = 0
for t in truth:
    for f in found:
        sides = {f["a"]["file"]: f["a"], f["b"]["file"]: f["b"]}
        if t["donor"] in sides and t["target"] in sides:
            s = sides[t["target"]]
            ov = min(s["end_line"], t["end"]) - max(s["start_line"], t["start"]) + 1
            if ov >= 0.5 * (t["end"] - t["start"] + 1):
                hit += 1
                break
inj = {t["donor"] for t in truth} | {t["target"] for t in truth}
other = [f for f in found if not ({f["a"]["file"], f["b"]["file"]} <= inj and
         any({f["a"]["file"], f["b"]["file"]} == {t["donor"], t["target"]} for t in truth))]
print(f"injected pairs={len(truth)} recall={hit / len(truth):.3f} total_fragments={len(found)} other_fragments={len(other)}")
