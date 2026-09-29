# duplicatecode

Static (LLM-free) detection of duplicate / similar code in Python and TypeScript (TSX),
aimed at catching an LLM re-implementing something that already exists. Input can be a git diff
checked against existing source.

- `crates/duplicatecode-engine` – Rust engine (tree-sitter based)
- `crates/duplicatecode-cli` – `duplicatecode` CLI
- `dataset/` – benchmark: coding tasks + diverging implementations (see `dataset/README.md`)

## Usage

```sh
cargo build --release
duplicatecode units src/                              # list extracted units
git diff origin/main | duplicatecode diff --repo .    # check added code against the repo
duplicatecode bench --dataset dataset [--file-level] [--mutations] [--negatives <other repo>]
```

## How it works

Units (functions, methods, classes, arrow-function components) are extracted with tree-sitter and
normalized (identifiers/literals abstracted; comments, docstrings and type annotations dropped).
Each pair gets several signals: token 4-/2-gram overlap, node-kind histogram, literals, API/attribute
names, name subwords, callees, and statement-level multiset/alignment scores. `combined` is a weighted
sum (0.6 structure, 0.2 name, 0.2 callees). Constructors and dunder methods are ignored by default.
An inverted index over k-grams keeps matching against large corpora fast.

## Benchmark findings (Haiku vs Sonnet implementations of the same 50 tasks)

- Ranking is good (right match is top-1 in ~70–95% of cases) but absolute detection at a 1% false-positive
  rate is modest for independently written code: ~35–50% (function level) of same-task pairs.
- Comparing whole files (which removes helper-splitting noise) gives 76–100% recall, mostly thanks to
  names; structure-only signals reach ~35–55% on loose/hard tasks. Statement-level features are roughly on
  par with token k-grams; a fitted logistic model does not beat the hand weights in cross-validation.
- Mechanical rewrites (rename, reorder, temp variable, logging, dead code) are handled well individually
  (86–100% of pairs ≥ 0.7); all combined drops the mean score to ~0.58, mostly because renames zero the name signal.
