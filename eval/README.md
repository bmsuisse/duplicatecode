# Evaluation: labeled clone groups

`duplicatecode eval-groups` scores every pair of units inside a dataset and reports how well the
`combined` score (and each individual signal) separates clones from non-clones:

- **AUC** – probability a random clone pair outscores a random non-clone pair
- **TPR@1% / TPR@5%** – share of clone pairs found at a fixed false-positive rate
- **top1** – for each file, is its best-scoring neighbour in the same group?
- **score** – mean over datasets of `(AUC + TPR@1%) / 2` for `combined`; the number to optimize

Layout: `<root>/<dataset>/<group>/<file>`; files in one group are clones. Datasets named `fn-*`
compare functions/classes (same-file pairs and boilerplate skipped), all others compare whole files.

| dataset | source | what it tests |
| --- | --- | --- |
| `python`, `javascript` | Project CodeNet (`iNeil77/CodeNet`), accepted submissions to one problem | independent solutions to the same task (hard, semantic clones) |
| `sql` | `gretelai/synthetic_text_to_sql` + `sqlglot` rewrites | optimizer rewrites and small edits of real queries |
| `react` | real `.tsx` components + `mutate` rewrites | rename/reorder/logging/dead-code robustness, false positives on look-alike components |
| `fn-python`, `fn-typescript` | `dataset/` (LLM re-implementations of 50 tasks) | the product path: function-level re-implementations |

Dev and holdout use disjoint queries (even vs odd SQL pages), components and tasks (fn: odd vs even
task number). The CodeNet sets are only seed-separated unless `--exclude-from` is used: measured overlap
without it was 13 of 60 Python and 4 of 60 JS problems. Tune on dev, confirm on holdout.

```sh
uv run eval/fetch_codenet.py --lang Python     --out eval/data/codenet
uv run eval/fetch_codenet.py --lang JavaScript --out eval/data/codenet --min-size 200
uv run eval/fetch_codenet.py --lang Python     --out eval/data/holdout --seed 0.99 \
    --exclude-from eval/data/codenet/python                              # same for JavaScript (.../javascript)
uv run eval/fetch_sql.py --queries 80 --seed 1 --parity 0 --out eval/data/codenet/sql
uv run eval/fetch_sql.py --queries 80 --seed 2 --parity 1 --out eval/data/holdout/sql
duplicatecode make-mutation-groups --src <repo with .tsx> --out eval/data/codenet/react --n 80 --skip 0
duplicatecode make-mutation-groups --src <repo with .tsx> --out eval/data/holdout/react --n 80 --skip 80
python3 eval/make_fn_data.py dataset eval/data/codenet eval/data/holdout

duplicatecode eval-groups                              # dev
duplicatecode eval-groups --root eval/data/holdout     # holdout
duplicatecode eval-groups --dump pairs.tsv             # per-pair feature vectors for offline fitting

# fragment-level clones (pasted blocks): inject renamed blocks into real functions, then score recall
python3 eval/make_fragment_data.py <python repo> /tmp/frag --pairs 60 --block 6 [--seed 1] [--force]
uv run --no-project --python 3.12 eval/make_fragment_data_ts.py <ts repo> /tmp/fragts --pairs 60 --block 6
# both write truth.json.txt and refuse a non-empty out dir without it (unless --force)
python3 eval/eval_fragments.py /tmp/frag duplicatecode --min-stmts 4 --min-tokens 30

# `duplicatecode find` (search by description): task descriptions as queries over implementation units
python3 eval/eval_find.py dataset duplicatecode [extra distractor dirs]
```

Embedder bake-off (needs Python 3.12: embed-anything has no wheels for the free-threaded 3.14):

```sh
uv run --no-project --python 3.12 eval/embed_bench.py <model-id> ... --dump-dev dev.tsv --dump-hold hold.tsv --out bench.json
uv run --no-project --python 3.12 eval/embed_server.py   # local /v1/embeddings on :8099; only allowlisted models (EMBED_MODELS adds more)
```

`eval/data/` is git-ignored. The React set is built from private code and stays local.

Caveats: the SQL and React sets are mutation-based (Type 1–3 clones), so they saturate and act as
regression guards; only CodeNet and the `fn-*` sets contain independently written solutions.
