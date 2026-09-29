# Benchmark dataset

Independent implementations of the same 50 tasks (25 Python, 25 TypeScript/TSX) by different LLMs,
used to test whether the detector finds "same thing, written differently".

- `tasks/` – strict specs (exact signature, edge cases, examples). Upper-bound test: names/signatures match by construction.
- `tasks-loose/` – 1–2 sentence casual requests, no names/signatures. Realistic test: models choose their own names.
- `tasks-hard/` – German business-problem descriptions of the same 50 concepts, avoiding technique vocabulary. Code must still be English.
- `impls/<model>/` – implementations of the strict specs (flat, file names given by the spec).
- `impls-loose/<model>/NN-slug/` – implementations of the loose requests (model-chosen names).

- `impls-hard/<model>/NN-slug/` – implementations of the hard prompts.

Models: `haiku` (claude-haiku-4-5), `sonnet` (claude-sonnet-5-5). Task NN is the same concept across all four sets,
so `(NN, language)` gives ground-truth duplicate groups.

All implementations pass `./check.sh <dir>` (ruff check + format, ty, biome) with no suppressions.
Known: `impls-hard/haiku` (26) and `impls-loose/haiku` (1) still have biome *warnings* (mostly `any`); exit code is 0.
Note: only lint/type-checked, not behavior-tested.
