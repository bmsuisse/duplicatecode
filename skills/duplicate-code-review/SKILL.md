---
name: duplicate-code-review
description: Find duplicated or near-duplicated code (Python, TypeScript/TSX) in a repo or monorepo folder, or check whether new code re-implements something that already exists, using the `duplicatecode` CLI. Use when asked to find/report duplicates, dead-weight copy-paste, "did we already write this", or to review a diff/PR for duplication. Cheap and fast; ranks candidates so you only read the promising ones.
---

# Duplicate code review with `duplicatecode`

`duplicatecode` is a static (LLM-free) detector. It parses code into functions/methods/classes/components/module-level values, normalizes them (identifiers, literals, comments, docstrings, type annotations, debug logging ignored; comprehensions read as loops), and scores pairs on structure, statements, literals, API calls and name similarity. **It finds candidates; you decide.** In tests it found ~90% of the duplicates a Sonnet-class reviewer found by reading, in a tenth of the tool calls, but only 30–50% of its raw pairs are real duplicates — the value you add is verification and pruning.

Binary: `duplicatecode` on PATH, or `<repo>/target/release/duplicatecode` (build: `cargo build --release`).

## Procedure (follow in order; ~10–25 tool calls is normal)

1. **Scan with `review`** (compact markdown: ranked groups, code preview, what differs):
   ```
   duplicatecode review <folder> [<folder2> …] --max-groups 60
   ```
   - Monorepo: pass the package folders, or the root. `.gitignore`, `node_modules`, hidden dirs, generated files (`*.gen.*`, `generated/`, `*.d.ts`) are skipped automatically. Add `--exclude '<glob>'` (repeatable) for anything else vendored.
   - Groups are tiered: **IDENTICAL** (same normalized body) > **NEAR-COPY** (score ≥ 0.6) > **SIMILAR**. Each group lists its members as `file:start-end kind name`, the signals of the closest pair, `differs:` (literals / calls only in one side) and a preview of the first member.
2. **Triage every IDENTICAL and NEAR-COPY group; sample the top of SIMILAR.** For each group decide, reading code if the preview is not enough (`duplicatecode show path/to/file.py:10-40` prints numbered lines, or use your Read tool):
   - **real duplicate** – would be one shared function/component/hook (copy-paste, or same job written slightly differently);
   - **partial** – shares a substantial sub-part worth extracting;
   - **noise** – similar only by convention → drop it (see below).
3. **Widen for recall** (do this; the first pass is precision-leaning):
   ```
   duplicatecode review <folder> --threshold 0.35 --min-name 0 --min-lines 3 --max-groups 80
   ```
   Skim only groups you have not seen. `--min-name 0` also lists look-alikes with unrelated names (finds renamed re-implementations, but noisier).
4. **Look for what a static tool cannot see** (same purpose, different code): pick 3–6 common helper families (format/parse/normalize/slug/debounce/retry/pagination/date/currency/auth/fetch-wrapper/query-client/logger setup…), `grep -rn "function <family>\|def <family>"` across the folder, and compare implementations by reading. Also check **module-level setup blocks** repeated across apps/packages (e.g. client/config construction).
5. **Report** (see format below). Verify each reported group by reading every member — never report from scores alone.

## Deciding what is noise (very common false positives)

Drop unless the bodies are truly identical *and* it is non-trivial:
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
