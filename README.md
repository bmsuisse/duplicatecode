# duplicatecode

Static by default (LLM-free, no network unless `--embed*` is used) detection of duplicate / similar code in Python, TypeScript/JavaScript (TSX/JSX), SQL and C#,
aimed at catching an LLM re-implementing something that already exists. Input can be a git diff
checked against existing source.

- `crates/duplicatecode-engine` – Rust engine (tree-sitter based)
- `crates/duplicatecode-cli` – `duplicatecode` CLI
- `dataset/` – benchmark: coding tasks + diverging implementations (see `dataset/README.md`)

## Install

```sh
uv tool install duplicatecode            # from PyPI (prebuilt binary, no Rust needed)
uvx duplicatecode scan .                 # or run without installing
curl -fsSL https://raw.githubusercontent.com/bmsuisse/duplicatecode/main/install.sh | sh   # standalone binary
```

Binaries for Linux/macOS/Windows (x86_64, plus aarch64 on Linux/macOS) are also attached to each
[GitHub Release](https://github.com/bmsuisse/duplicatecode/releases). Releases are automatic: bump
`version` in `[workspace.package]` in `Cargo.toml`, merge to `main`, and CI tags and publishes it.

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

## More ways to look

```sh
duplicatecode fragments packages/            # copied blocks inside different functions (4+ identical statements)
duplicatecode scan . --pairs --explain       # say what differs: literals, calls, lines (what to parameterize)
duplicatecode find "retry with exponential backoff" src/   # does something like this already exist? (needs embeddings)
```

- `fragments` indexes windows of consecutive normalized statements, so a block pasted into another
  function is found even when the surrounding functions differ. On an injection benchmark (renamed
  blocks of real functions pasted into others) it finds 93-97% of blocks of 6+ statements; see
  `eval/REPORT.md`. Tune with `--min-stmts` / `--min-tokens`; constructors and dunders are ignored.
- `--explain` adds, per pair, the literals and calls only one side has and the source lines with no
  counterpart ("16 of 18/17 statements shared").
- `find` embeds the query and every unit and ranks by similarity. With MiniLM, 50 task descriptions
  against 168 implementation units rank the right one first 94% of the time; with 1,379 unrelated real
  units mixed in, 82% first and 96% in the top 3 (every query within the top 10).
- Default thresholds depend on the profile: `copies` keeps its tuned 0.6 (`scan`), 0.4 (`diff`), 0.45
  (`review`); `reimpl` scores live lower, so it defaults to 0.35 / 0.28 / 0.30, which correspond to roughly
  0.1% / 1% false-positive rates on unrelated code in the benchmarks. With `--embed` the `reimpl`
  defaults rise by 0.07 because blended scores of unrelated code rise too.

## Embeddings (bring your own key)

Two optional signals, both off by default (the detector stays static and offline unless you ask).

> **Privacy warning.** `--embed openai`, `--embed cohere`, and `--embed-code` / `--embeddings` pointed at any
> hosted endpoint (OpenAI, Cohere, Azure, ...) **upload the source text of your units** to that provider
> (`--embeddings` sends identifier names only). `find` embeds **every unit under the given paths**, so
> `find ... --embed openai` uploads all of it. The local presets (`minilm`, `qwen3`, `potion`) are meant for a
> server on loopback (default `127.0.0.1:8099`; `DUPLICATECODE_EMBED_ENDPOINT` overrides it): pointing that
> variable at a remote host sends text there too. Do not use hosted embeddings on code you may not share.

- `--embeddings` embeds identifier *names* (cheap) as an extra name-similarity signal.
- `--embed <preset>` is the short form (`minilm`, `qwen3`, `potion`, `openai`, `cohere`; `qwen3` is about 5x slower than `minilm` on CPU, so use it with a GPU or prefer `minilm`) and implies
  `--embed-code`. Model ids go into the cache key, so vectors of different models never mix.
- `--embed-code` embeds the *whole text of every unit* (function, class, file, SQL statement) and blends
  the cosine into the score (`--embed-weight`, default 0.35, others scaled by 1 - weight). It finds
  re-implementations that share no tokens. On function-level LLM re-implementations it lifted Python from
  0.70 to 0.89 on a held-out half; see `eval/REPORT.md` for the numbers and caveats. Vectors are cached per
  unit text in `~/.cache/duplicatecode/embeddings.bin`, so a rescan only embeds what changed.

Credentials come from the environment:

```sh
# OpenAI
export OPENAI_API_KEY=sk-...                      # optional: OPENAI_EMBEDDING_MODEL (default text-embedding-3-small)
# Cohere (native /v2/embed)
export COHERE_API_KEY=...                         # optional: COHERE_EMBEDDING_MODEL (default embed-v4.0)
# OpenAI-compatible server (vLLM, Ollama, LiteLLM, ...)
export OPENAI_BASE_URL=http://localhost:11434/v1 OPENAI_API_KEY=anything OPENAI_EMBEDDING_MODEL=nomic-embed-text
# Azure AI Foundry (key, or `az login` if no key is set)
export AZURE_AI_FOUNDRY_ENDPOINT=https://<res>.services.ai.azure.com
export AZURE_AI_FOUNDRY_API_KEY=... AZURE_AI_FOUNDRY_EMBEDDING_DEPLOYMENT=text-embedding-3-small
# Azure OpenAI: AZURE_OPENAI_ENDPOINT / AZURE_OPENAI_API_KEY / AZURE_OPENAI_EMBEDDING_DEPLOYMENT
# Fully local, no key: start the server once (CPU; it loads whichever model a request names)
#   uv run --no-project --python 3.12 eval/embed_server.py
#   duplicatecode scan . --embed minilm          # or: qwen3, potion  (default endpoint 127.0.0.1:8099)
# Hosted presets: --embed openai (OPENAI_API_KEY), --embed cohere (COHERE_API_KEY)

duplicatecode embed-test fetchUser getUser     # check credentials
duplicatecode scan . --embed-code              # whole-unit embeddings
duplicatecode scan . --embeddings              # names only
```

## API

### Command line

| command | what it does |
| --- | --- |
| `scan <paths>` | similar units inside one or more folders; `--pairs`, `--explain`, `--json`, `--fail-on-found` |
| `diff` | check code added in a git diff (stdin or `--diff`) against the repo (`--repo`) |
| `review` | ranked candidates for a diff, for a human or an agent to judge |
| `fragments <paths>` | copied blocks of identical statements inside different functions |
| `find "<description>" <paths>` | existing units closest to a plain-language description (needs embeddings) |
| `units <path>` / `show file:10-40` | list extracted units / print a unit's source |
| `embed-test <names...>` | check the embedding provider and calibrate `--embed-floor` |
| `eval-groups`, `bench`, `make-mutation-groups` | evaluation (see `eval/README.md`) |

Shared options: `--profile copies|reimpl`, `--threshold`, `--min-tokens`, `--min-lines`, `--exclude`, `--skip-tests`,
`--cross-file`, `--json`. Embedding options (all off by default): `--embed <preset>`, `--embed-code`,
`--embed-weight`, `--embed-max-chars`, `--embeddings`, `--embed-cache`, `--embed-dims`.

```sh
duplicatecode scan . --profile reimpl --min-name 0 --embed minilm --pairs --explain
duplicatecode fragments src/ --min-stmts 4 --min-tokens 30 --cross-file --json
duplicatecode find "retry with exponential backoff" src/ --top 5 --json
duplicatecode diff --repo . < change.diff --embed openai
```

JSON output:

- `find --json`: `[{score, file, name, kind, start_line, end_line}]`
- `fragments --json`: `[{a, b, statements, tokens}]` with `a`/`b` = `{file, unit, start_line, end_line, coverage}`
- `scan --pairs --json`: `[{query, candidate, scores}]`; with `--explain`: `[{match, explanation}]`
- `scan --json` (groups): `[{score, units: [{file, name, kind, start_line, end_line}]}]`

`scan --fail-on-found` exits with status 1 when anything is reported, for CI.

### Rust library

The engine crate `duplicatecode-engine` is what the CLI uses:

```rust
use duplicatecode_engine::embed::{embed_unit_code, EmbedConfig, EmbeddingCache};
use duplicatecode_engine::explain::explain;
use duplicatecode_engine::fragments::{find_fragments, FragmentOptions};
use duplicatecode_engine::index::Weights;
use duplicatecode_engine::{find_matches, load_units_with, Corpus, MatchOptions};

let mut units = load_units_with(Path::new("src/"), &[]);

// optional: embed every unit; units without a vector simply skip the embedding term
let cfg = EmbedConfig::from_env(None).ok_or("no embedding provider configured")?;
let mut cache = EmbeddingCache::load(&EmbeddingCache::default_path());
embed_unit_code(&mut units, &cfg, &mut cache, 3000)?;

// pairs: queries against a corpus
let weights = Weights::default().with_embed(0.35); // 0.0 = static only
let corpus = Corpus::new(units.clone());
let pairs = find_matches(&units, &corpus, MatchOptions { weights, threshold: 0.42, ..Default::default() });

// copied blocks, and what differs inside a pair
let blocks = find_fragments(&units, FragmentOptions::default());
println!("{}", explain(&units[0], &units[1]).summary());
```

### Not built yet

Planned, not available today: a `.duplicatecode.toml` with an `[embed]` section; `--embed-optional` (warn and
fall back to the static score when the provider is unreachable; today an unreachable provider is an error);
`--allow-upload` (required for the hosted presets, which send source text out); a `duplicatecode[embed]` Python
extra that starts the local server on demand; an `Embedder` trait so providers can plug in without HTTP.

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

### Fresh held-out check (copies profile, threshold 0.6 = the `copies` default for `scan`)

A third, untouched sample of 57 pairs (nothing was tuned on it; the `reimpl` profile has different defaults, 0.35 for `scan`, and is not covered by this sample): 30% true duplicates, 47% incl. partial
(OneSales 0/24, MDMApp 11/24, CCMT2 6/9 true). The 50%/76% above was optimistic because it was measured on the
pairs used to choose the weights. Test code is about half of the noise but also holds real copies
(25% true either way), so it is kept by default; `--skip-tests` drops it.

### Re-implementations of existing helpers (`reimpl-eval`)

42 real helper functions from the three repos were described neutrally (no names/code) and re-implemented from scratch
by Haiku and Sonnet without seeing the repo. Fraction where the original is found (copies profile) and where the best
*other* match also crosses the threshold:

| threshold | Sonnet found | Haiku found | both | other-code alarm |
|---|---|---|---|---|
| 0.3 | 67% | 60% | 63% | 21% |
| 0.4 | 62% | 33% | 48% | 8% |
| 0.5 | 45% | 14% | 30% | 1% |
| 0.6 | 24% | 5% | 14% | 0% |

So with the `copies` profile `diff` (checking new code) defaults to threshold 0.4 while `scan` (existing copies) defaults to 0.6; the `reimpl` profile has its own lower defaults (see above). About half of
independent re-implementations are caught; the rest are genuinely different code. The structure-heavy `reimpl`
profile is not better on this test.

### Head-to-head: LLM-only vs LLM + CLI (MDMApp, OneSales)

Four Sonnet agents (report only, ~80 tool calls, max 30 groups) hunted duplicates in two repos: two with plain
read/grep, two with this CLI. All distinct groups (82) were then judged blind by independent agents that read the code.

| | groups | true dup. | precision (true / incl. partial) | pooled recall (true) | tool calls |
|---|---|---|---|---|---|
| MDMApp, LLM only | 25 | 15 | 60% / 80% | 60% | ~27–33 |
| MDMApp, with CLI | 26 | 20 | 77% / 92% | 80% | 7 |
| OneSales, LLM only | 28 | 10 | 36% / 71% | 67% | ~33–42 |
| OneSales, with CLI | 21 | 10 | 48% / 86% | 67% | 9 |

Only 18 of 82 groups were found by both, so the approaches are complementary. The agents' reports are capped at 30
groups, so the raw tool is a better measure of recall: `scan --threshold 0.6` (the `copies` defaults) finds 38 of the 40 judged
true duplicates (25/25 MDMApp, 13/15 OneSales) and 23 of the 25 that the LLM-only agents found independently.
What it still misses: a differently-written picker function (same purpose, different code) and a formatFileSize
variant with different constants. Fixed after this test: tiny same-name exact copies (`min_tokens` 20 -> 8, near-exact
same-name rule) and multi-line module-level values such as `export const queryClient = new QueryClient({...})`.

### Cheap model + CLI + skill (Haiku) vs Sonnet (MDMApp, OneSales)

Same task and judging as above (blind judges, pooled recall against all distinct confirmed true duplicates: 37 in MDMApp,
18 in OneSales). `review` + `skills/duplicate-code-review/SKILL.md` were built from what the first runs missed.
"cost index" = tokens x relative price (Haiku assumed 1/3 of Sonnet per token; estimate).

| repo | approach | groups | precision true / incl. partial | recall | tokens | cost index |
|---|---|---|---|---|---|---|
| MDMApp | Sonnet, no CLI | 25 | 60% / 80% | 41% | 86k | 259 |
| MDMApp | Sonnet + CLI | 29 | 79% / 93% | 54% | 59k | 177 |
| MDMApp | **Haiku + CLI + skill** (r1 / r2) | 31 / 40 | 90% / 90%  ·  70% / 82% | 65% / **70%** | 86k / 86k | **86** |
| OneSales | Sonnet, no CLI | 28 | 36% / 71% | 56% | 131k | 393 |
| OneSales | Sonnet + CLI | 21 | 52% / 90% | 61% | 62k | 187 |
| OneSales | **Haiku + CLI + skill** (r1 / r2) | 11 / 30 | 82% / 91%  ·  33% / 67% | 50% / 50% | 88k / 74k | **74–88** |

r1 = first skill (top-60 groups only); r2 = skill with `--brief` paging. Read: on MDMApp Haiku + CLI beats Sonnet alone on
recall (70% vs 41%) and precision at about a third of the cost; on OneSales it is at par with Sonnet alone on recall
(50% vs 56%, one group) with better-or-equal precision at about a fifth of the cost, but below Sonnet + CLI. Broadening (r2)
buys recall at the price of precision on OneSales. Small samples; judges and finders are the same model family.

Round 3 (after `LIKELY-NOISE` tagging and the line-diff view; hard cap 30 groups): MDMApp 29 groups, precision 69% / 86%,
recall 51%, 69k tokens; OneSales 12 groups, precision 75% / 83%, recall 42%, 89k tokens. Recall pooled against 41 (MDMApp)
and 19 (OneSales) confirmed duplicates. Run-to-run variation of Haiku (59–63% / 51% on MDMApp, 42–47% on OneSales) is as
large as the differences between skill/tool versions, so the tag and diff view are not shown to help measurably; averaged over
three runs Haiku + CLI + skill lands at ~58% (MDMApp) and ~45% (OneSales) recall versus 37% / 53% for Sonnet alone and
49% / 63% for Sonnet + CLI, at roughly a fifth to a third of the cost of Sonnet alone. Note the 30-group cap is itself a ceiling:
the confirmed pool holds 41 real duplicates in MDMApp.

### Uncapped comparison (same 80-group cap, ~120-call budget for every arm) — final

Pooled confirmed true duplicates: 48 (MDMApp), 35 (OneSales); every group anyone reported was judged blind by reading the code.

| repo | arm | groups | precision true / incl. partial | recall | tokens | cost index (Haiku = 1/3 price) |
|---|---|---|---|---|---|---|
| MDMApp | Sonnet, no CLI | 65 | 42% / 65% | 65% | 174k | 522 |
| MDMApp | Sonnet + CLI + skill | 69 | 62% / 86% | 77% | 72k | 214 |
| MDMApp | Haiku + CLI + skill | 59 | 73% / 86% | 67% | 102k | **101** |
| OneSales | Sonnet, no CLI | 64 | 30% / 56% | 54% | 153k | 459 |
| OneSales | Sonnet + CLI + skill | 80 | 32% / 71% | 66% | 90k | 270 |
| OneSales | Haiku + CLI + skill | 47 | 38% / 68% | 40% | 85k | **84** |
| both | tool list only, IDENTICAL+NEAR-COPY, minus `LIKELY-NOISE` | 173 / 278 | (judged subset: 66% / 85% MDMApp, 38% / 64% OneSales) | **88% / 83%** | 0 | 0 |

- With the same budget Haiku + CLI matches Sonnet alone on MDMApp (67% vs 65% recall, better precision) at ~1/5 of the cost;
  on OneSales it is below (40% vs 54%) at ~1/5.5 of the cost. Sonnet + CLI is best on recall in both.
- The unfiltered tool list already covers 83–88% of the pool: the agents' job is pruning, and their lower recall comes
  from what they choose to report (they judge from brief lines and read few members), not from what the tool misses.
  Precision of the raw list is unknown beyond the judged subset (which is biased towards groups an agent reported).
- Caveats: judges and finders are the same model family; the pool only contains duplicates someone found; single runs
  per arm (Haiku varies by +-10 points between runs).
