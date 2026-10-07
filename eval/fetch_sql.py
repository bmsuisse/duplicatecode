# /// script
# requires-python = ">=3.10"
# dependencies = ["sqlglot==30.21.0"]
# ///
"""Build labeled SQL clone groups from gretelai/synthetic_text_to_sql.

No public dataset ships SQL equivalence labels, so each real query is expanded into semantically
equivalent variants with sqlglot (alias/CTE renames, formatting + keyword case, reordered AND
conjuncts, subqueries lifted into CTEs). Variants of one query form a group; other queries are
negatives. This measures robustness to Type 1-3 rewrites, not independent re-implementation.

    uv run eval/fetch_sql.py --queries 80 --seed 1 --out eval/data/codenet/sql
    uv run eval/fetch_sql.py --queries 80 --seed 2 --parity 1 --out eval/data/holdout/sql

Rows are fetched in 100-row pages. Dev draws from even page indexes (`--parity 0`), holdout from odd ones
(`--parity 1`), so the two sets can never share a source row whatever the seeds are.
"""
import argparse
import json
import pathlib
import random
import time
import urllib.error
import urllib.request

import sqlglot
from sqlglot import exp
from sqlglot.optimizer.eliminate_subqueries import eliminate_subqueries
from sqlglot.optimizer.merge_subqueries import merge_subqueries

API = ("https://datasets-server.huggingface.co/rows?dataset=gretelai/synthetic_text_to_sql"
       "&config=default&split=train&offset={o}&length=100")
WANT = {"subqueries", "multiple_joins", "aggregation", "window functions", "CTEs", "set operations"}


CACHE = pathlib.Path("eval/data/_sql_rows")


def rows(offset: int) -> list[dict]:
    f = CACHE / f"{offset}.json"
    if f.exists():
        return json.loads(f.read_text())
    for attempt in range(8):
        try:
            with urllib.request.urlopen(API.format(o=offset), timeout=60) as r:
                data = [x["row"] for x in json.load(r)["rows"]]
            CACHE.mkdir(parents=True, exist_ok=True)
            f.write_text(json.dumps(data))
            time.sleep(1.5)
            return data
        except urllib.error.HTTPError as e:
            if e.code != 429:
                raise
            time.sleep(5 * (attempt + 1))
        except (urllib.error.URLError, TimeoutError, ConnectionError) as e:
            print(f"offset {offset} attempt {attempt}: {e}")
            time.sleep(5 * (attempt + 1))
    raise RuntimeError("rate limited")


def rename_aliases(tree: exp.Expression, tag: str) -> exp.Expression:
    t = tree.copy()
    mapping = {}
    for node in t.find_all(exp.TableAlias):
        old = node.name
        if old and old not in mapping:
            mapping[old] = f"{tag}{len(mapping)}"
    for node in t.find_all(exp.CTE, exp.Table, exp.TableAlias, exp.Column):
        if isinstance(node, exp.CTE):
            if node.alias in mapping:
                node.set("alias", exp.TableAlias(this=exp.to_identifier(mapping[node.alias])))
        elif isinstance(node, exp.TableAlias):
            if node.name in mapping:
                node.set("this", exp.to_identifier(mapping[node.name]))
        elif isinstance(node, exp.Table):
            if not node.db and node.name in mapping:
                node.set("this", exp.to_identifier(mapping[node.name]))
        elif isinstance(node, exp.Column):
            if node.table in mapping:
                node.set("table", exp.to_identifier(mapping[node.table]))
    return t


def schema_of(context: str) -> dict:
    schema: dict[str, dict[str, str]] = {}
    for st in sqlglot.parse(context):
        if isinstance(st, exp.Create) and isinstance(st.this, exp.Schema):
            cols = {c.name: c.args["kind"].sql() if c.args.get("kind") else "text"
                    for c in st.this.expressions if isinstance(c, exp.ColumnDef)}
            if cols:
                schema[st.this.this.name] = cols
    return schema


def drop_predicate(tree: exp.Expression) -> exp.Expression | None:
    t = tree.copy()
    for w in t.find_all(exp.Where):
        if isinstance(w.this, exp.And):
            w.set("this", w.this.this)
            return t
    return None


def add_projection(tree: exp.Expression) -> exp.Expression | None:
    t = tree.copy()
    if isinstance(t, exp.Select) and t.expressions and not any(e.find(exp.Star) for e in t.expressions):
        return t.select(t.expressions[0].copy().unalias(), append=True)
    return None


def tweak(tree: exp.Expression) -> exp.Expression | None:
    t = tree.copy()
    for lit in t.find_all(exp.Literal):
        lit.set("this", str(int(lit.this) + 7) if not lit.is_string and lit.this.isdigit() else lit.this + "x")
        break
    else:
        return None
    return t.limit(10) if isinstance(t, exp.Select) and not t.args.get("limit") else t


def variants(sql: str, context: str) -> dict[str, str]:
    from sqlglot.optimizer import optimize
    tree = sqlglot.parse_one(sql)
    out = {"orig": sql.strip().rstrip(";") + ";"}
    try:
        out["opt"] = optimize(tree.copy(), schema=schema_of(context)).sql(pretty=True) + ";"
    except Exception:
        return {}
    for name, fn in (("pred", drop_predicate), ("proj", add_projection), ("tweak", tweak)):
        v = fn(tree)
        if v is not None:
            out[name] = v.sql(pretty=True) + ";"
    out["alias"] = rename_aliases(tree, "q").sql(pretty=True) + ";"
    try:
        e = drop_predicate(tree) or tree
        out["opt_pred"] = optimize(e.copy(), schema=schema_of(context)).sql(pretty=True) + ";"
    except Exception:
        pass
    return out


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--queries", type=int, default=80)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--parity", type=int, choices=(0, 1), default=0, help="page parity: 0 = dev, 1 = holdout")
    ap.add_argument("--out", type=pathlib.Path, required=True)
    a = ap.parse_args()
    rnd = random.Random(a.seed)
    pages = list(range(a.parity, 990, 2))  # 100-row pages of ~99k rows; parity keeps dev/holdout disjoint
    rnd.shuffle(pages)
    pool: dict[int, dict] = {}
    for page in pages[:14]:
        for r in rows(page * 100):
            try:
                pool.setdefault(int(r["id"]), r)
            except (TypeError, ValueError):
                continue
    pool_rows = list(pool.values())
    rnd.shuffle(pool_rows)
    written = skipped = 0
    for r in pool_rows:
        if written >= a.queries:
            break
        if r["sql_complexity"] not in WANT or len(r["sql"]) < 120:
            continue
        try:
            v = variants(r["sql"], r["sql_context"])
        except Exception:  # noqa: BLE001  sqlglot cannot parse/optimize some queries
            skipped += 1
            continue
        if not v:
            skipped += 1
            continue
        if len(set(v.values())) < 4:
            continue
        d = a.out / f"q{int(r['id'])}"
        d.mkdir(parents=True, exist_ok=True)
        for name, sql in v.items():
            (d / f"{name}.sql").write_text(sql + "\n", encoding="utf-8")
        written += 1
    print(f"wrote {written} query groups to {a.out} ({skipped} queries skipped: sqlglot failed)")


if __name__ == "__main__":
    main()
