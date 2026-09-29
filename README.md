# duplicatecode

Static (LLM-free) detection of duplicate / similar code in Python and TypeScript (TSX),
aimed at catching an LLM re-implementing something that already exists. Input can be a git diff
checked against existing source.

- `crates/duplicatecode-engine` – Rust engine (tree-sitter based)
- `crates/duplicatecode-cli` – `duplicatecode` CLI
- `dataset/` – benchmark: coding tasks + diverging implementations (see `dataset/README.md`)
