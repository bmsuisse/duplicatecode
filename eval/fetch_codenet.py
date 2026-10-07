# /// script
# requires-python = ">=3.10"
# dependencies = ["duckdb==1.5.6"]
# ///
"""Sample labeled clone groups from Project CodeNet (HF: iNeil77/CodeNet).

Accepted submissions to the same problem solve the same task, so they are behavioural clones;
submissions to different problems are negatives. Writes `<out>/<lang>/<problem>/<submission>.<ext>`.

    uv run eval/fetch_codenet.py --lang Python --problems 60 --per-problem 8 --out eval/data/codenet
    uv run eval/fetch_codenet.py --lang Python --out eval/data/holdout --seed 0.99 --exclude-from eval/data/codenet/python

Without `--exclude-from`, dev and holdout differ only by random seed and can share problems (measured: 13 of
60 Python and 4 of 60 JavaScript problems overlapped). `--exclude-from DIR` skips every problem whose id is a
directory name in DIR (e.g. the dev set's `<lang>` dir), making the holdout disjoint from dev.
"""
import argparse
import pathlib
import re
import shutil
import time
import urllib.error
import urllib.request

import duckdb

EXT = {"Python": "py", "JavaScript": "js", "TypeScript": "ts"}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--lang", default="Python", choices=EXT)
    ap.add_argument("--problems", type=int, default=60)
    ap.add_argument("--per-problem", type=int, default=8)
    ap.add_argument("--min-size", type=int, default=300)
    ap.add_argument("--max-size", type=int, default=3000)
    ap.add_argument("--shards", type=int, default=2, help="Python parquet shards to download")
    ap.add_argument("--seed", type=float, default=0.42)
    ap.add_argument("--exclude-from", type=pathlib.Path, help="dir of problem folders whose ids are excluded from sampling")
    ap.add_argument("--out", type=pathlib.Path, default=pathlib.Path("eval/data/codenet"))
    a = ap.parse_args()

    excl = sorted(p.name for p in a.exclude_from.iterdir() if p.is_dir()) if a.exclude_from else []
    con = duckdb.connect()
    con.execute("SELECT setseed(?)", [a.seed])
    base = f"https://huggingface.co/api/datasets/iNeil77/CodeNet/parquet/{a.lang}/train"
    cache = a.out / "_cache"
    cache.mkdir(parents=True, exist_ok=True)
    shards = []
    for i in (range(1, 1 + a.shards) if a.lang == "Python" else [0]):
        dest = cache / f"{a.lang}-{i}.parquet"
        if not dest.exists():  # remote range reads get rate-limited (429); download once instead
            for attempt in range(5):
                part = dest.with_name(dest.name + ".part")  # rename on success: no truncated shard looks complete
                try:
                    with urllib.request.urlopen(f"{base}/{i}.parquet", timeout=120) as r, open(part, "wb") as fh:
                        shutil.copyfileobj(r, fh)
                    part.replace(dest)
                    break
                except (urllib.error.URLError, TimeoutError, ConnectionError) as e:
                    print(f"shard {i} attempt {attempt}: {e}")
                    part.unlink(missing_ok=True)
                    time.sleep(5 * (attempt + 1))
        if dest.exists():
            shards.append(str(dest))
    rows = con.execute(
        f"""
        WITH ok AS (
          SELECT p_id, s_id, u_id, code,
                 row_number() OVER (PARTITION BY p_id, u_id ORDER BY random()) AS per_user
          FROM read_parquet($shards)
          WHERE status = 'Accepted' AND length(code) BETWEEN $lo AND $hi
            AND p_id NOT IN (SELECT unnest($excl::VARCHAR[]))
        ), distinct_users AS (SELECT * FROM ok WHERE per_user = 1),
        big AS (SELECT p_id FROM distinct_users GROUP BY p_id HAVING count(*) >= $n
                ORDER BY random() LIMIT $p),
        ranked AS (
          SELECT d.*, row_number() OVER (PARTITION BY d.p_id ORDER BY random()) AS rk
          FROM distinct_users d JOIN big USING (p_id)
        )
        SELECT p_id, s_id, code FROM ranked WHERE rk <= $n
        """,
        {"shards": shards, "lo": a.min_size, "hi": a.max_size, "n": a.per_problem, "p": a.problems, "excl": excl},
    ).fetchall()

    root = a.out / a.lang.lower()
    safe = re.compile(r"[A-Za-z0-9_.-]+")
    rows = [r for r in rows if safe.fullmatch(r[0]) and safe.fullmatch(r[1]) and r[0] not in (".", "..")]
    for p, s, code in rows:
        d = root / p
        d.mkdir(parents=True, exist_ok=True)
        (d / f"{s}.{EXT[a.lang]}").write_text(code, encoding="utf-8")
    print(f"{a.lang}: wrote {len(rows)} files for {len({r[0] for r in rows})} problems to {root}")


if __name__ == "__main__":
    main()
