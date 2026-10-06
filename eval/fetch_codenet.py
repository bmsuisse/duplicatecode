# /// script
# requires-python = ">=3.10"
# dependencies = ["duckdb"]
# ///
"""Sample labeled clone groups from Project CodeNet (HF: iNeil77/CodeNet).

Accepted submissions to the same problem solve the same task, so they are behavioural clones;
submissions to different problems are negatives. Writes `<out>/<lang>/<problem>/<submission>.<ext>`.

    uv run eval/fetch_codenet.py --lang Python --problems 60 --per-problem 8 --out eval/data/codenet
"""
import argparse
import pathlib
import time
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
    ap.add_argument("--out", type=pathlib.Path, default=pathlib.Path("eval/data/codenet"))
    a = ap.parse_args()

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
                try:
                    urllib.request.urlretrieve(f"{base}/{i}.parquet", dest)
                    break
                except Exception as e:  # noqa: BLE001
                    print(f"shard {i} attempt {attempt}: {e}")
                    dest.unlink(missing_ok=True)
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
        ), distinct_users AS (SELECT * FROM ok WHERE per_user = 1),
        big AS (SELECT p_id FROM distinct_users GROUP BY p_id HAVING count(*) >= $n
                ORDER BY random() LIMIT $p),
        ranked AS (
          SELECT d.*, row_number() OVER (PARTITION BY d.p_id ORDER BY random()) AS rk
          FROM distinct_users d JOIN big USING (p_id)
        )
        SELECT p_id, s_id, code FROM ranked WHERE rk <= $n
        """,
        {"shards": shards, "lo": a.min_size, "hi": a.max_size, "n": a.per_problem, "p": a.problems},
    ).fetchall()

    root = a.out / a.lang.lower()
    for p, s, code in rows:
        d = root / p
        d.mkdir(parents=True, exist_ok=True)
        (d / f"{s}.{EXT[a.lang]}").write_text(code, encoding="utf-8")
    print(f"{a.lang}: wrote {len(rows)} files for {len({r[0] for r in rows})} problems to {root}")


if __name__ == "__main__":
    main()
