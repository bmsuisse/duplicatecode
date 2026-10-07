# /// script
# requires-python = ">=3.10"
# dependencies = ["tree-sitter", "tree-sitter-typescript"]
# ///
"""TypeScript/TSX variant of make_fragment_data.py: paste a renamed block of statements from one real
function into another. Same output layout and truth file, scored by eval_fragments.py.

    uv run --no-project eval/make_fragment_data_ts.py <ts repo> <out dir> [--pairs 60] [--block 6] [--seed 1]
"""
import json, pathlib, random, re, sys

import tree_sitter_typescript as tst
from tree_sitter import Language, Parser

args = sys.argv[1:]
src_root, out = pathlib.Path(args[0]), pathlib.Path(args[1])
opt = lambda name, d: int(args[args.index(name) + 1]) if name in args else d
pairs, block, seed = opt("--pairs", 60), opt("--block", 6), opt("--seed", 1)
rnd = random.Random(seed)
parser = Parser(Language(tst.language_tsx()))
STOP = {"return_statement", "break_statement", "continue_statement", "yield_expression", "throw_statement"}
OK = {"lexical_declaration", "expression_statement", "if_statement", "for_statement", "for_in_statement"}


def walk(n):
    yield n
    for c in n.children:
        yield from walk(c)


funcs = []  # (text, [statement nodes relative offsets])
for path in sorted(src_root.rglob("*.ts*")):
    s = str(path)
    if path.suffix not in (".ts", ".tsx") or any(x in s for x in ("node_modules", ".test.", ".d.ts", "/.worktrees/", "/.claude/", "/generated/", "/dist/")):
        continue
    data = path.read_bytes()
    tree = parser.parse(data)
    for n in walk(tree.root_node):
        if n.type != "function_declaration":
            continue
        body = n.child_by_field_name("body")
        stmts = [c for c in body.named_children if c.type != "comment"] if body else []
        if len(stmts) >= 10 and n.end_byte - n.start_byte < 6000:
            funcs.append((data[n.start_byte:n.end_byte].decode("utf8", "ignore"), n.start_byte, data, stmts))
rnd.shuffle(funcs)
print(f"{len(funcs)} candidate functions")


def pick_block(item):
    _, base, data, stmts = item
    for _ in range(40):
        i = rnd.randrange(0, len(stmts) - block + 1)
        run = stmts[i:i + block]
        if all(s.type in OK and not any(d.type in STOP for d in walk(s)) for s in run):
            return data[run[0].start_byte:run[-1].end_byte].decode("utf8", "ignore"), run
    return None


def rename(text, run):
    names = []
    for s in run:
        for d in walk(s):
            if d.type == "variable_declarator":
                nm = d.child_by_field_name("name")
                if nm is not None and nm.type == "identifier":
                    names.append(nm.text.decode())
    for k, nm in enumerate(dict.fromkeys(names)):
        text = re.sub(rf"\b{re.escape(nm)}\b", f"v_{k}", text)
    return text


out.mkdir(parents=True, exist_ok=True)
for f in out.glob("*.ts*"):
    f.unlink()
chosen = []
for i in range(0, len(funcs) - 1, 2):
    if len(chosen) >= pairs:
        break
    picked = pick_block(funcs[i])
    if picked:
        chosen.append((funcs[i], funcs[i + 1], picked))
truth = []
for n, (donor, target, (btext, run)) in enumerate(chosen):
    (out / f"d{n}.tsx").write_text(donor[0] + "\n")
    ttext, tbase, tdata, tstmts = target
    at = rnd.randrange(1, len(tstmts))
    pos = tstmts[at].start_byte - tbase
    indent = " " * tstmts[at].start_point[1]
    new = ttext[:pos] + rename(btext, run) + "\n" + indent + ttext[pos:]
    (out / f"t{n}.tsx").write_text(new + "\n")
    start = new[:pos].count("\n") + 1
    truth.append({"donor": f"d{n}.tsx", "target": f"t{n}.tsx", "start": start, "end": start + btext.count("\n")})
for j, f in enumerate(funcs[2 * len(chosen):][:400]):
    (out / f"x{j}.tsx").write_text(f[0] + "\n")
(out / "truth.json.txt").write_text(json.dumps(truth))
print(f"wrote {len(truth)} injected pairs + {min(400, len(funcs[2*len(chosen):]))} distractor functions to {out}")
