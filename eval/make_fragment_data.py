"""Fragment-clone benchmark: paste a renamed block of statements from one real function into another.

    python3 eval/make_fragment_data.py <python repo> <out dir> [--pairs 60] [--block 6] [--seed 1] [--force]

Writes `d<n>.py` = donor function, `t<n>.py` = target with the renamed block injected, `x<j>.py` = distractor
functions, and `truth.json.txt` listing (donor file, target file, injected line range in the target).
A non-empty <out dir> without a `truth.json.txt` is refused unless --force is given.
"""
import argparse, ast, copy, json, pathlib, random, sys

SKIP_DIRS = {"node_modules", ".venv", "venv", ".worktrees", ".claude", "generated", "dist", "build", ".git"}


def excluded(rel: pathlib.PurePath) -> bool:
    """Path filters on directory names / file name patterns relative to the repo root."""
    parts = rel.parts
    if any(p in SKIP_DIRS or p.lower().startswith("test") or p == "__tests__" for p in parts[:-1]):
        return True
    name = parts[-1]
    return name.startswith("test_") or name.endswith("_test.py") or name == "conftest.py"


def simple(st: ast.stmt) -> bool:
    return not isinstance(st, (ast.Return, ast.Break, ast.Continue, ast.Yield, ast.Global, ast.Nonlocal,
                               ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Import, ast.ImportFrom)) \
        and not any(isinstance(n, (ast.Return, ast.Yield, ast.YieldFrom, ast.Await)) for n in ast.walk(st))

def pick_block(f: ast.FunctionDef, rnd: random.Random, block: int):
    body = f.body
    for _ in range(40):
        i = rnd.randrange(0, max(1, len(body) - block))
        run = body[i:i + block]
        if len(run) == block and all(simple(s) for s in run):
            return run
    return None

def rename(stmts):
    names = {n.id for st in stmts for n in ast.walk(st) if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Store)}
    m = {n: f"v_{k}" for k, n in enumerate(sorted(names))}
    class R(ast.NodeTransformer):
        def visit_Name(self, node):
            node.id = m.get(node.id, node.id)
            return node
    return [R().visit(copy.deepcopy(st)) for st in stmts]

def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("src_root", type=pathlib.Path)
    ap.add_argument("out", type=pathlib.Path)
    ap.add_argument("--pairs", type=int, default=60)
    ap.add_argument("--block", type=int, default=6)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--force", action="store_true", help="allow writing into a non-empty dir without truth.json.txt")
    a = ap.parse_args()
    src_root, out, pairs, block = a.src_root, a.out, a.pairs, a.block
    rnd = random.Random(a.seed)

    funcs = []
    for path in sorted(src_root.rglob("*.py")):
        if excluded(path.relative_to(src_root)):
            continue
        try:
            tree = ast.parse(path.read_text())
        except (SyntaxError, UnicodeDecodeError):
            continue
        for node in ast.walk(tree):
            if isinstance(node, ast.FunctionDef) and len(node.body) >= 10:
                funcs.append(node)
    rnd.shuffle(funcs)
    print(f"{len(funcs)} candidate functions")
    if not funcs:
        sys.exit(f"no candidate functions (>= 10 statements) found under {src_root}")

    if out.exists() and any(out.iterdir()) and not (out / "truth.json.txt").exists() and not a.force:
        sys.exit(f"{out} is not empty and has no truth.json.txt; refusing to clean it (use --force)")
    out.mkdir(parents=True, exist_ok=True)
    chosen = []
    for i in range(0, len(funcs) - 1, 2):
        if len(chosen) >= pairs:
            break
        donor, target = funcs[i], funcs[i + 1]
        run = pick_block(donor, rnd, block)
        if run:
            chosen.append((donor, target, run))
    if not chosen:
        sys.exit("no injected pairs could be built (need at least 2 functions and a block of simple statements)")
    for f in out.glob("*.py"):
        if f.name == "truth.json.txt" or f.name[0] in "dtx" and f.stem[1:].isdigit():
            f.unlink()
    truth = []
    for n, (donor, target, run) in enumerate(chosen):
        (out / f"d{n}.py").write_text(ast.unparse(donor) + "\n")
        tgt = ast.parse(ast.unparse(target))
        fn = tgt.body[0]
        at = rnd.randrange(1, len(fn.body) + 1)
        fn.body[at:at] = rename(run)
        text = ast.unparse(tgt) + "\n"
        (out / f"t{n}.py").write_text(text)
        re = ast.parse(text).body[0].body
        truth.append({"donor": f"d{n}.py", "target": f"t{n}.py", "start": re[at].lineno, "end": re[at + block - 1].end_lineno})
    # distractors: the other functions, one per file
    for j, f in enumerate(funcs[2 * len(chosen):][:400]):
        (out / f"x{j}.py").write_text(ast.unparse(f) + "\n")
    (out / "truth.json.txt").write_text(json.dumps(truth))
    print(f"wrote {len(truth)} injected pairs + {min(400, len(funcs[2*len(chosen):]))} distractor functions to {out}")


if __name__ == "__main__":
    main()
