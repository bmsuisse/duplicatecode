# Benchmark dataset

Independent implementations of the same 50 tasks (25 Python, 25 TypeScript/TSX) by different LLMs,
used to test whether the detector finds "same thing, written differently".

- `tasks/` – strict specs (exact signature, edge cases, examples). Upper-bound test: names/signatures match by construction.
- `tasks-loose/` – 1–2 sentence casual requests, no names/signatures. Realistic test: models choose their own names.
- `impls/<model>/` – implementations of the strict specs (flat, file names given by the spec).
- `impls-loose/<model>/NN-slug/` – implementations of the loose requests (model-chosen names).

Models: `haiku` (claude-haiku-4-5), `sonnet` (claude-sonnet-5-5). Task NN is the same concept across all four sets,
so `(NN, language)` gives ground-truth duplicate groups.

All implementations pass `./check.sh <dir>` (ruff check + format, ty, biome) with no suppressions.
Note: only lint/type-checked, not behavior-tested.
