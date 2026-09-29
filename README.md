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
duplicatecode scan packages/ services/api           # find duplicate groups inside one or more folders (monorepo)
duplicatecode scan . --exclude '**/generated/**' --skip-tests --fail-on-found   # CI-style run
duplicatecode scan . --pairs --json                  # machine-readable pairs instead of groups
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

## Normalization

Before comparing, debug output (`print`, `logging.*`, `console.*`) is dropped; Python comprehensions are
read as the equivalent loop; `x += 1` as `x = x + 1`; a temp variable returned right after its definition
is inlined; operators are canonicalized (`===`/`==`/`is`, `and`/`&&`, `None`/`null`, `const`/`let`).
Mutation benchmark (`bench --mutations`): each of rename, statement swap, temp variable, logging, dead code and
loop-to-comprehension keeps >= 93% of pairs at score >= 0.7; all combined 40% (renames zero the name signal).

## Real-repo validation (hand-judged)

Self-scans of three internal repositories (Python + TypeScript), 189 pairs judged by reading both units
(TRUE_DUP / PARTIAL / BOILERPLATE / FALSE):

- Raw scores are a weak signal: only ~23% of pairs above 0.55 were true duplicates, 40% incl. partial; below
  0.65 almost none. Real duplicates were almost all exact copies with matching names.
- Dominant false alarms: react-query/fetch wrappers differing by endpoint, per-entity CRUD endpoints, one-line
  repository delegators, DTO/model classes, per-file test fixtures, generated clients.
- Filters added from this: field-only classes, constructors/dunders, generated files, tiny units (unless
  near-identical with the same name), test code (unless identical), minimum name similarity (0.3).
- `--profile copies` (default) re-weights signals for copy-paste (name 0.36, statements 0.27, literals 0.18);
  leave-one-repo-out AUC improved on all three repos (0.71/0.69/0.95 -> 0.75/0.79/0.96). At threshold 0.6 the
  judged sample gives ~50% true duplicates and ~76% incl. partial while keeping ~70% of the true ones.
  These numbers are partly in-sample (weights and filters were chosen on the same pairs) — re-judge a fresh sample
  before trusting them. A structure-heavy `--profile reimpl` exists for renamed re-implementations
  (use with `--min-name 0`) but has no real-repo precision data yet.

### Fresh held-out check (copies profile, threshold 0.6)

A third, untouched sample of 57 pairs (nothing was tuned on it): 30% true duplicates, 47% incl. partial
(OneSales 0/24, MDMApp 11/24, CCMT2 6/9 true). The 50%/76% above was optimistic because it was measured on the
pairs used to choose the weights. Test code is about half of the noise but also holds real copies
(25% true either way), so it is kept by default; `--skip-tests` drops it.
