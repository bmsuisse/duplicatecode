# Duplicate-code detection: what was tested, what helped, what is left

Branch `autoresearch/dedup-quality`. All numbers come from `duplicatecode eval-groups` (see
`eval/README.md`). Tune on **dev**, confirm on **holdout** (disjoint problems / queries / components /
tasks). Headline score = mean over datasets of `(AUC + TPR@1%FPR) / 2` for the combined score.

## Result

| dataset | what it is | before (dev / holdout) | after (dev / holdout) |
| --- | --- | --- | --- |
| python | CodeNet, independent solutions per problem (file level) | 0.559 / 0.510 | **0.693 / 0.659** |
| javascript | CodeNet, independent solutions (file level) | 0.391 / 0.368 | **0.503 / 0.497** |
| sql | real queries + sqlglot optimizer rewrites and edits | 0.836 / n.a. | **0.989 / 0.993** |
| react | real TSX components + mechanical rewrites | 0.992 / 0.980 | 0.991 / 0.980 (guard) |
| fn-python | LLM re-implementations, function level | 0.645 / 0.647 | **0.698 / 0.706** |
| fn-typescript | LLM re-implementations, function level | 0.678 / 0.669 | **0.823 / 0.728** |

"before" = the original reimpl-profile weights. For python/javascript that is the untouched original
engine; for fn-* it is the original weights on the current (JS/TS-normalized) tokens, because the original
build cannot run `eval-groups`; SQL did not exist, so its "before" is the new parser with the old weights.
The two CodeNet sets alone went from **0.475 / 0.439 to 0.598 / 0.578** (python+javascript mean, dev /
holdout); the six-dataset headline from **0.749 / 0.741** (after the function-level fix, old fn weights)
to **0.783 / 0.761**.

The default CLI profile (`copies`) is untouched: a scan of a real Python repo gives the identical 209 pairs
with identical scores before and after. The gains apply to the `reimpl` profile, whole-file units, SQL and
JS/TS normalization.

## Experiments (journal: `autoresearch-dedup-quality.tsv`)

Kept (each re-measured on holdout):

| # | change | effect |
| --- | --- | --- |
| 1 | reweight reimpl profile toward loose 2-grams / statements / literals | dev +0.031, holdout +0.039 |
| 2 | IDF-weighted n-gram Jaccard (rare n-grams count more) | +0.008 / +0.005 |
| 4 | blank unused top-level defs and imports before comparing files (templates, helper libraries) | +0.013 / +0.011 |
| 5 | keep `print` / `console.log` in whole-file units (it is the program's result there) | +0.032 / +0.040 |
| 7 | metric-searched blend weights | +0.011 / +0.012 |
| 10 | SQL support + its own weights (loose n-grams, tables/columns, literals) | SQL 0.836 -> 0.989 |
| 12 | prune unused pure constants (`MOD = 10**9+7`, `INF = float('inf')`) | python +0.010 / +0.021 |
| 13 | inline a once-called zero-arg `main()` wrapper | python +0.015 / +0.007 |
| 15 | ignore `;` in JS/TS (ASI) | js +0.015 / +0.016 (also fn-typescript +0.016 / +0.018) |
| 16 | ignore block braces in JS/TS | js +0.003 / +0.008 |
| 17 | ignore `sys.setrecursionlimit` and `input = sys.stdin.readline` alias lines | python +0.010 / +0.012 |
| 19 | function-level weights retuned on LLM re-implementations, containment as 11th feature, absent features (no literals/calls on either side) renormalized away | fn-python +0.05, fn-typescript +0.04..0.12 |

Discarded (kept out because they did not generalize or were noise):

| # | idea | why not |
| --- | --- | --- |
| 3 | TF-IDF token-bag cosine as a new feature | collinear with n-gram Jaccard; weight 0 in the best blend |
| 6 | keep library call/method names in the token stream | dev +0.005 but holdout -0.009 |
| 8 | n-gram size 3 / 5 / 6 instead of 4 | holdout 0.5435 / 0.5410 / 0.5374 < 0.5465 |
| 14a | treat `var` like `let`/`const` for dead-constant pruning | +0.0003 / -0.0004 (noise) |
| 14b | inline `function Main(input)` wrapper in JS | neutral |

### A mistake worth recording

Experiments 1-17 were tuned on whole-file CodeNet scripts only. That search zeroed the name and callee
weights (file names are meaningless there) and **regressed the original function-level benchmark**
(`bench`: Python loose top-1 69% -> 62%, TypeScript strict 95% -> 89%). It was caught by comparing against a
build of the original `main`; the fix gives whole-file units their own weights and adds the LLM
re-implementation dataset to the loop. Function-level `bench` top-1 (combined) after the fix, vs original:
Python strict 94 -> 99, strict~hard 79 -> 83; TypeScript strict 95 -> 97, strict~hard 85 -> 88; a few hard
rows are 2-3 points lower (Python hard~hard 70 -> 67, loose~hard 70 -> 67; TypeScript hard~hard 85 -> 83).

## Offline ceilings measured (not shipped)

| model on the same pair features, trained on dev, scored on holdout | python | javascript | fn-python | fn-typescript |
| --- | --- | --- | --- | --- |
| current hand-set blend (shipped) | 0.659 | 0.497 | 0.706 | 0.728 |
| logistic regression | 0.684 | 0.544 | 0.743 | 0.695 |
| gradient-boosted trees | 0.690 | 0.573 | 0.751 | 0.667 |

so a learned re-ranker over the current signals would add roughly +0.03 (python), +0.05 to +0.08
(javascript) and +0.04 (fn-python), and nothing on fn-typescript (the 12-task function set is too small to
fit 11 weights without overfitting). Most of the remaining headroom is in information the syntactic
signals do not carry.

## Embeddings

Whole-unit embeddings (Qwen3-Embedding-0.6B via `embed-anything`, CPU, first 3000 chars of each unit),
cosine similarity of the two units, measured on the same pairs as the static score (dev data only; the
blend weight lambda was chosen on half of the groups and tested on the other half):

| dataset | static score | cosine alone | blend (lambda) | held-out half: static -> blend |
| --- | --- | --- | --- | --- |
| fn-python (LLM re-implementations, function level) | 0.700 | **0.831** | 0.50: 0.831 | 0.699 -> **0.891** |
| fn-typescript (same) | **0.802** | 0.724 | 0.20: 0.828 | 0.612 -> 0.673 |
| python (CodeNet, whole files) | 0.689 | 0.979 | 0.50: 0.976 | 0.717 -> 0.975 |
| javascript (CodeNet, whole files) | 0.503 | 0.957 | 0.50: 0.932 | 0.497 -> 0.966 |

How much to trust it:

- **The CodeNet rows are inflated and should not be quoted.** Cosine alone reaches 0.98 / 0.96, far above
  anything structural. Explicit problem ids or URLs in comments explain only about 4% of the files
  (21/480 Python, 5/480 JS), so the likely cause is training-set contamination (CodeNet is a standard
  code-model corpus). I did not run a comment-stripped re-embedding.
- **The fn-* rows are the clean test**: the LLM implementations were written for this repo, so the model
  cannot have seen them. Python improves a lot (held-out half 0.70 -> 0.89); TypeScript only a little
  (0.61 -> 0.67) and cosine alone is worse than the static score there.
- Embeddings see docstrings and comments, which state the intent. That is legitimate for finding
  re-implementations, but it is not a pure code-structure signal, and the static detector deliberately ignores
  them.
- Only dev data was embedded, there are 12-13 tasks per language, and lambda is a single parameter, so the
  direction is reliable but the exact gain is not.
- Cost: about 2.5 s per file on 24 CPU cores for a 0.6B model, i.e. roughly 40 minutes for 960 files; a
  GPU or a hosted endpoint is needed for real repositories. Vectors are cacheable per unit text.

### Embedder bake-off (`eval/embed_bench.py`, `embed-anything`, CPU, clean data only)

Each model embeds the fn-* units and SQL files of dev and holdout (1507 texts, mean 371 chars). "cosine"
is the model alone, "blend" is `0.65 * static + 0.35 * cosine` (the shipped `--embed-weight`), scores are
`(AUC + TPR@1%FPR) / 2`; static baseline is 0.70 / 0.70 / 0.80 / 0.71. Timings were taken with four
benchmarks sharing 24 cores, so read them as relative.

| model | dim | ms/text | cosine: fn-py dev/hold, fn-ts dev/hold | blend: same | mean blend gain |
| --- | --- | --- | --- | --- | --- |
| sentence-transformers/all-MiniLM-L6-v2 | 384 | 248 | 0.84 0.81 / 0.68 0.90 | 0.81 0.78 / 0.80 0.85 | **+0.083** |
| Qwen/Qwen3-Embedding-0.6B | 1024 | 1168 | 0.83 0.80 / 0.72 0.90 | 0.80 0.77 / 0.82 0.85 | +0.081 |
| BAAI/bge-small-en-v1.5 | 384 | 436 | 0.83 0.78 / 0.56 0.86 | 0.79 0.76 / 0.77 0.81 | +0.052 |
| jinaai/jina-embeddings-v2-base-en | 768 | 471 | 0.74 0.79 / 0.52 0.79 | 0.73 0.74 / 0.77 0.76 | +0.018 |
| minishlab/potion-base-8M (static) | 256 | ~1 | 0.75 0.68 / 0.47 0.74 | 0.76 0.74 / 0.70 0.77 | +0.013 |

SQL is saturated (about 1.0 for every model), so it does not discriminate. Models that `embed-anything`
cannot load in this build: `nomic-ai/modernbert-embed-base`, `Alibaba-NLP/gte-modernbert-base`
("Model not supported"), `jinaai/jina-code-embeddings-0.5b`, `BAAI/bge-m3` (same), and
`jinaai/jina-embeddings-v2-base-code` (missing tensor). The code-specific Jina models therefore could not
be tested.

Proposal:

- **Default: `sentence-transformers/all-MiniLM-L6-v2`.** Statistically tied with Qwen3-0.6B on this data at
  a quarter of the time per text, 22M parameters and 384-dim vectors (small cache). Caveat: it truncates at
  256 word pieces, so it sees only the start of long units.
- **Quality option: `Qwen/Qwen3-Embedding-0.6B`.** Same measured gain, 8k-token context, better on
  TypeScript dev cosine (0.72 vs 0.68); use it where units are long and CPU time is not a concern.
- **Instant, no-model-server option: `minishlab/potion-base-8M`.** About 1 ms per text and still positive
  (+0.013), useful as a cheap always-on signal.
- **Hosted:** OpenAI `text-embedding-3-small` and Cohere `embed-v4.0` are wired in but not measured (no
  key); they should be added to the same bake-off before choosing a hosted default.

The top two are within noise on 25 tasks; the ranking among them is not established. A larger,
uncontaminated function-level set is the prerequisite for a firmer default.

Recommendation: add unit-level embeddings as an **optional** signal behind a flag (blend about 0.3-0.5 for
Python), keep the static path as the default, and re-measure on a larger uncontaminated function-level set
before choosing a default lambda.


## What could still be optimized

1. **Embeddings as an optional signal** (see above for the measured value). Design: keep the static path
   as the default; `--embeddings` already supports OpenAI, OpenAI-compatible and Azure AI Foundry
   endpoints for names; extend it to whole-unit text, cache vectors in the existing flat file, and
   add one blend weight. A local `embed` extra needs a Python helper the CLI can call, because the wheel is
   a Rust binary; a CUDA build of `embed-anything` is only worthwhile once the GPU driver loads.
2. **Ship IDF in the scan path.** IDF is worth +0.014 headline (mostly whole files; +0.007 on fn-python), but
   `Corpus::best_for` scores without it. The k-gram prefilter assumes unweighted Jaccard, so IDF needs
   stop-gram postings or a looser bound. Small gain, moderate work.
3. **A bigger function-level benchmark.** The `fn-*` sets are 25 tasks split odd/even, which is small
   enough that 11 weights can overfit. Independent re-implementations of real repo functions (the
   `reimpl-eval` idea) in Python, TypeScript/React and SQL are the most valuable missing data.
4. **React-specific normalization.** The React set is saturated by mechanical rewrites, so it says nothing
   about independent components. JSX-aware normalization (prop destructuring vs `props.x`, hook order,
   class-name string literals, wrapper components) is untested.
5. **SQL dialect coverage and scope analysis.** `tree-sitter-sequel` is generic/Postgres-flavoured; Spark
   (`values:key[0].data::string`, `left anti join`, backticks) parses with error nodes. Alias/CTE
   resolution is name-based; a real scope pass and column lineage would make renames and CTE-vs-subquery
   rewrites exact. The synthetic SQL set saturates (0.99), so real labelled pairs from
   `Fabricks.Runtime` are the next test.
6. **JavaScript is the weakest language (0.50).** Input handling idioms (`readFileSync`, `process.stdin`
   events, `readline`) and loop styles (`for` / `forEach` / `map` / `reduce`) differ per author; canonical
   forms for these would help, and a loop-to-iterator normalization like the Python comprehension one.
7. **Learned re-ranker** (trees, see the ceilings table) compiled into the engine, gated behind a flag so
   the default stays interpretable.
8. **Scan speed with the new weights.** The reimpl profile's structural weight is small, so the k-gram
   prefilter prunes less; measure scan time on large repos before making it a default. SQL scoring with
   structural weight 0 compares every same-family unit.
9. **GPU.** The RTX 4080 is present but the NVIDIA kernel module is not loaded and `sudo` is denied, so CPU
   was used for all embedding runs (about 2.5 s per file with a 0.6B model). `eval/setup-gpu.sh` needs
   someone with root.

## Limitations of these numbers

- CodeNet "clones" are accepted solutions to one problem, so they are behavioural clones, but a few
  problems have trivially short solutions that look alike across problems.
- SQL and React sets are mutation-based; they check robustness, not independent re-implementation.
- Whole-file units are not the main product path; the function-level sets are the ones that matter most
  and are the smallest.
- Dev-tuned weights were confirmed on holdout, but holdout files come from the same sources.
