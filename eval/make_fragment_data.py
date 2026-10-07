"""Fragment-clone benchmark: paste a renamed block of statements from one real function into another.

    python3 eval/make_fragment_data.py <python repo> <out dir> [--pairs 60] [--block 6] [--seed 1]

Writes every pool function as its own file (`f<k>.py`), `t<i>.py` = target with the block injected,
and `truth.json` listing (donor file, target file, injected line range in the target).
"""
import ast, json, pathlib, random, sys

args = sys.argv[1:]
src_root, out = pathlib.Path(args[0]), pathlib.Path(args[1])
opt = lambda name, d: int(args[args.index(name) + 1]) if name in args else d
pairs, block, seed = opt("--pairs", 60), opt("--block", 6), opt("--seed", 1)
rnd = random.Random(seed)

def simple(st: ast.stmt) -> bool:
    return not isinstance(st, (ast.Return, ast.Break, ast.Continue, ast.Yield, ast.Global, ast.Nonlocal,
                               ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Import, ast.ImportFrom)) \
        and not any(isinstance(n, (ast.Return, ast.Yield, ast.YieldFrom, ast.Await)) for n in ast.walk(st))

funcs = []
for path in sorted(src_root.rglob("*.py")):
    s = str(path)
    if any(x in s for x in ("test", "/.venv/", "node_modules", "/.worktrees/", "/.claude/")):
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

def pick_block(f: ast.FunctionDef):
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
    import copy
    return [R().visit(copy.deepcopy(st)) for st in stmts]

out.mkdir(parents=True, exist_ok=True)
for f in out.glob("*.py"):
    f.unlink()
truth, k = [], 0
used = set()
chosen = []
for i in range(0, len(funcs) - 1, 2):
    if len(chosen) >= pairs:
        break
    donor, target = funcs[i], funcs[i + 1]
    run = pick_block(donor)
    if run:
        chosen.append((donor, target, run))
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
