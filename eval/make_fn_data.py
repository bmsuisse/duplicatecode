"""Adapt the LLM re-implementation dataset (dataset/) to eval-groups' function-level layout.

Ground truth is the task number: implementations of the same task by different models/prompts are
clones. Output `<root>/fn-<lang>/<NN>/<set>-<model>-<file>`; odd tasks go to the dev root, even
tasks to the holdout root, so tuning never sees the holdout tasks.

    python3 eval/make_fn_data.py dataset eval/data/codenet eval/data/holdout
"""
import pathlib
import re
import shutil
import sys

LANG = {".py": "python", ".ts": "typescript", ".tsx": "typescript"}


def main(dataset: pathlib.Path, dev: pathlib.Path, hold: pathlib.Path) -> None:
    target = {}
    for md in sorted((dataset / "tasks").glob("*.md")):
        m = re.search(r"\*\*Target file:\*\*\s*`([^`]+)`", md.read_text())
        if m:
            target[m.group(1)] = md.name[:2]
    for root in (dev, hold):
        for d in root.glob("fn-*"):
            shutil.rmtree(d)
    n = 0
    skipped: list[pathlib.Path] = []
    seen: dict[pathlib.Path, pathlib.Path] = {}

    def put(task: str, src: pathlib.Path, tag: str) -> None:
        nonlocal n
        lang = LANG.get(src.suffix)
        if not lang:
            skipped.append(src)
            return
        root = dev if int(task) % 2 else hold
        out = root / f"fn-{lang}" / task
        out.mkdir(parents=True, exist_ok=True)
        dest = out / f"{tag}-{src.name}"
        if dest in seen:
            sys.exit(f"duplicate target key {dest}: {seen[dest]} and {src}")
        seen[dest] = src
        shutil.copyfile(src, dest)
        n += 1

    for model in sorted((dataset / "impls").iterdir()):
        for f in sorted(model.iterdir()):
            if f.name in target:
                put(target[f.name], f, f"strict-{model.name}")
            else:
                skipped.append(f)
    for kind in ("impls-loose", "impls-hard"):
        for model in sorted((dataset / kind).iterdir()):
            for task in sorted(p for p in model.iterdir() if p.is_dir()):
                for f in sorted(task.iterdir()):
                    put(task.name[:2], f, f"{kind[6:]}-{model.name}")
    if skipped:
        print(f"skipped {len(skipped)} files (no task mapping or unsupported extension):")
        for f in skipped:
            print(f"  {f}")
    print(f"wrote {n} files")


if __name__ == "__main__":
    main(*map(pathlib.Path, sys.argv[1:4]))
