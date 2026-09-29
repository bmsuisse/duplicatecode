#!/usr/bin/env bash
# Usage: ./check.sh impls/<model>   -- runs ruff, ty (python) and biome (typescript)
set -u
dir="${1:?dir}"; rc=0
cd "$(dirname "$0")"
if [ -n "$(find "$dir" -name '*.py' -print -quit)" ]; then
  uvx ruff check "$dir" || rc=1
  uvx ruff format --check "$dir" || rc=1
  uvx ty check "$dir" || rc=1
fi
if [ -n "$(find "$dir" \( -name '*.ts' -o -name '*.tsx' \) -print -quit)" ]; then
  npx --yes @biomejs/biome check "$dir" || rc=1
fi
exit $rc
