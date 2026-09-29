---
name: duplicate-code-review
description: Find duplicated or near-duplicated code (Python, TypeScript/TSX) in a repo or monorepo folder, or check whether new code re-implements something that already exists, using the `duplicatecode` CLI. Use when asked to find/report duplicates, dead-weight copy-paste, "did we already write this", or to review a diff/PR for duplication. Cheap and fast; ranks candidates so you only read the promising ones.
---

# Duplicate code review with `duplicatecode`

`duplicatecode` is a static (LLM-free) detector. It parses code into functions/methods/classes/components/module-level values, normalizes them (identifiers, literals, comments, docstrings, type annotations, debug logging ignored; comprehensions read as loops), and scores pairs on structure, statements, literals, API calls and name similarity. **It finds candidates; you decide.** In tests it found ~90% of the duplicates a Sonnet-class reviewer found by reading, in a tenth of the tool calls, but only 30–50% of its raw pairs are real duplicates — the value you add is verification and pruning.

Binary: `duplicatecode` on PATH, or `<repo>/target/release/duplicatecode` (build: `cargo build --release`).

## Procedure (follow in order; ~15–40 tool calls is normal — use your budget, a repo usually has 50–300 groups)

1. **Get the full map cheaply with `--brief`** (one line per group, ranked IDENTICAL > NEAR-COPY > SIMILAR):
   ```
   duplicatecode review <folder> [<folder2> …] --brief --skip-tests --max-groups 150
   ```
   The header tells you how many groups exist per tier; page with `--offset N` until you have seen **every IDENTICAL and NEAR-COPY group** (`--tier identical` / `--tier near` filter). Each line: `G<n> TIER score x<members> `name` [hints]: file:lines | file:lines …`.
   - Monorepo: pass the package folders or the root. `.gitignore`, `node_modules`, hidden dirs and generated files (`*.gen.*`, `generated/`, `*.d.ts`) are skipped automatically; add `--exclude '<glob>'` (repeatable) for other vendored code.
   - Lines tagged `LIKELY-NOISE` (parametrized twins that differ only in names/literals, thin delegators) are sorted last inside their tier; skip them unless the members share a name *and* are not trivial wrappers.
2. **Open the promising groups in detail** (preview, signals, what differs, and a line **diff of the two closest members** — small diff = copy with small edits, large diff = only similar in shape) without `--brief`, e.g. `review <folder> --offset <n-1> --max-groups 1`, or just read the members (`duplicatecode show path/to/file.py:10-40`, or your Read tool). A group is promising when the members share a name, or a non-trivial body, or the same job. Skip groups that look like the noise patterns below without opening them.
3. **Decide per group** after reading every member:
   - **real duplicate** – would be one shared function/component/hook (copy-paste, or same job written slightly differently);
   - **partial** – shares a substantial sub-part worth extracting;
   - **noise** – similar only by convention → drop it.
4. **Widen for recall** (the first pass is precision-leaning):
   ```
   duplicatecode review <folder> --brief --threshold 0.35 --min-name 0 --min-lines 3 --max-groups 150
   ```
   Skim only groups you have not seen. `--min-name 0` also lists look-alikes with unrelated names (finds renamed re-implementations, noisier). Then repeat step 1 without `--skip-tests` if tests matter (identical helpers copied between test files are real but low priority).
5. **Look for what a static tool cannot see** (same purpose, different code): pick 4–8 common helper families (format/parse/normalize/slug/debounce/retry/pagination/date/currency/size/auth/fetch-wrapper/query-client/logger setup/pick/label…), `grep -rn "function <family>\|def <family>"` across the folder, and compare implementations by reading. Also check **module-level setup blocks** repeated across apps/packages (client/config construction).
6. **Report** (format below). Verify each reported group by reading every member — never report from scores alone.

## Deciding what is noise (very common false positives)

Drop unless the bodies are truly identical *and* it is non-trivial (the tool already tags the commonest shapes `LIKELY-NOISE`):
- thin API/fetch/react-query wrappers that differ only by endpoint, key or entity;
- per-entity CRUD/list endpoints, repository one-liners that delegate to `fetch_all`/`fetch_one`;
- DTO / model / TypedDict / Pydantic classes with similar field lists;
- test functions and fixtures that only share arrange-act-assert shape (identical *helpers* copied between test files are still worth reporting as low priority);
- argparse `main()` skeletons, shadcn/className wrapper components, i18n label maps, story/preview files;
- generated or vendored code.

Strong positives: identical or near-identical helpers copied between packages (`getInitials`, `formatNumber`, `useDebounced`, `apiFetch`, `_wait_for_port`, `queryClient` setup), same-name functions with the same body, create/update twins that duplicate large blocks, and a locally re-implemented helper when a shared one exists. **Same name + same body = almost always real.**

## Other commands

- `duplicatecode scan <folders> [--pairs] [--json] [--skip-tests] [--fail-on-found]` – raw groups/pairs (CI-friendly; exits 1 when found with `--fail-on-found`).
- `git diff origin/main | duplicatecode diff --repo . [--threshold 0.4]` – does the *new* code re-implement something existing? (default threshold 0.4; about half of independent re-implementations score ≥ 0.4 — lower to 0.3 if you must not miss, and read the hits.)
- `--profile reimpl --min-name 0` – structure-heavy scoring for renamed rewrites (experimental; noisier).
- `--embeddings` – adds semantic name similarity (Azure AI Foundry / Azure OpenAI / OpenAI-compatible; needs `AZURE_AI_FOUNDRY_ENDPOINT` + `AZURE_AI_FOUNDRY_API_KEY` or `az login`; vectors cached in `~/.cache/duplicatecode/embeddings.bin`). Use when helpers have different names for the same idea. `duplicatecode embed-test a b` checks setup.

## Report format

Unless told otherwise, write a list of groups, best first, each with ≥ 2 units:

```json
[{"kind": "exact-copy|near-copy|same-purpose",
  "confidence": "high|medium|low",
  "why": "one line: what is duplicated and what differs",
  "units": [{"file": "<path relative to repo root>", "start_line": 10, "end_line": 24, "name": "getInitials"}]}]
```

`exact-copy` = identical after renames; `near-copy` = copy with small edits; `same-purpose` = different code, same job. Put real copy-paste before partials. Don't pad the list with noise to look thorough; do include low-confidence real ones.

## Rules of thumb

- Trust IDENTICAL + same name; be skeptical of high scores where names differ and both bodies are short.
- Tiny helpers (< 6 lines) are only reported by the tool when near-identical *and* same-named — that is by design; report them if they are real.
- Line ranges come from the tool; keep them as printed.
- If output is huge, re-run with `--skip-tests`, a narrower folder, or a higher `--threshold` rather than reading everything.
