# /// script
# requires-python = ">=3.10,<3.13"
# dependencies = ["tree-sitter==0.26.0", "tree-sitter-typescript==0.23.2"]
# ///
"""TypeScript/TSX variant of make_fragment_data.py: paste a renamed block of statements from one real
function into another. Same output layout (`d<n>`, `t<n>`, `x<j>`, `truth.json.txt`), scored by eval_fragments.py.
`.ts` files are parsed with the TypeScript grammar and `.tsx` with the TSX grammar; output files are always
written as `.tsx` (TSX is a superset for the statement blocks used here, and the scorer only needs one suffix).

    uv run --no-project --python 3.12 eval/make_fragment_data_ts.py <ts repo> <out dir> [--pairs 60] [--block 6] [--seed 1] [--force]

A non-empty <out dir> without a `truth.json.txt` is refused unless --force is given.
"""
import argparse, json, pathlib, random, re, sys

import tree_sitter_typescript as tst
from tree_sitter import Language, Parser

STOP = {"return_statement", "break_statement", "continue_statement", "yield_expression", "throw_statement"}
OK = {"lexical_declaration", "expression_statement", "if_statement", "for_statement", "for_in_statement"}
SKIP_DIRS = {"node_modules", ".worktrees", ".claude", "generated", "dist", "build", ".git"}


def excluded(rel: pathlib.PurePath) -> bool:
    """Path filters on directory names / file name patterns relative to the repo root."""
    parts = rel.parts
    if any(p in SKIP_DIRS for p in parts[:-1]):
        return True
    name = parts[-1]
    return ".test." in name or name.endswith(".d.ts")


def walk(n):
    yield n
    for c in n.children:
        yield from walk(c)


def pick_block(item, rnd, block):
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


def find_funcs(src_root: pathlib.Path, parsers: dict) -> list:
    funcs = []  # (text, start_byte, data, statement nodes)
    for path in sorted(src_root.rglob("*.ts*")):
        if path.suffix not in parsers or excluded(path.relative_to(src_root)):
            continue
        data = path.read_bytes()
        tree = parsers[path.suffix].parse(data)
        for n in walk(tree.root_node):
            if n.type != "function_declaration":
                continue
            body = n.child_by_field_name("body")
            stmts = [c for c in body.named_children if c.type != "comment"] if body else []
            if len(stmts) >= 10 and n.end_byte - n.start_byte < 6000:
                funcs.append((data[n.start_byte:n.end_byte].decode("utf8", "ignore"), n.start_byte, data, stmts))
    return funcs


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
    parsers = {".ts": Parser(Language(tst.language_typescript())), ".tsx": Parser(Language(tst.language_tsx()))}
    funcs = find_funcs(src_root, parsers)
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
        picked = pick_block(funcs[i], rnd, block)
        if picked:
            chosen.append((funcs[i], funcs[i + 1], picked))
    if not chosen:
        sys.exit("no injected pairs could be built (need at least 2 functions and a block of simple statements)")
    for f in out.glob("*.tsx"):
        if f.stem[0] in "dtx" and f.stem[1:].isdigit():
            f.unlink()
    truth = []
    for n, (donor, target, (btext, run)) in enumerate(chosen):
        (out / f"d{n}.tsx").write_text(donor[0] + "\n")
        ttext, tbase, tdata, tstmts = target
        at = rnd.randrange(1, len(tstmts))
        # tree-sitter offsets are UTF-8 byte offsets: slice bytes, not the decoded str
        tb = ttext.encode("utf8")
        pos = tstmts[at].start_byte - tbase
        indent = " " * tstmts[at].start_point[1]  # byte column (indentation is ASCII)
        ins = (rename(btext, run) + "\n" + indent).encode("utf8")
        new = (tb[:pos] + ins + tb[pos:]).decode("utf8", "ignore")
        (out / f"t{n}.tsx").write_text(new + "\n")
        start = tb[:pos].count(b"\n") + 1
        truth.append({"donor": f"d{n}.tsx", "target": f"t{n}.tsx", "start": start, "end": start + btext.count("\n")})
    for j, f in enumerate(funcs[2 * len(chosen):][:400]):
        (out / f"x{j}.tsx").write_text(f[0] + "\n")
    (out / "truth.json.txt").write_text(json.dumps(truth))
    print(f"wrote {len(truth)} injected pairs + {min(400, len(funcs[2*len(chosen):]))} distractor functions to {out}")


if __name__ == "__main__":
    main()
