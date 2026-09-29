"""Recursive dictionary deep merge."""

import copy
from typing import Any, Literal


def _merge_lists(a: list[Any], b: list[Any], strategy: str) -> list[Any]:
    if strategy == "replace":
        return copy.deepcopy(b)
    combined = copy.deepcopy(a + b)
    if strategy == "concat":
        return combined
    unique: list[Any] = []
    for item in combined:
        if item not in unique:
            unique.append(item)
    return unique


def deep_merge(
    base: dict[str, Any],
    override: dict[str, Any],
    *,
    list_strategy: Literal["replace", "concat", "unique"] = "replace",
) -> dict[str, Any]:
    """Return a new dict combining ``base`` and ``override`` recursively."""
    result: dict[str, Any] = {}
    for key, value in base.items():
        result[key] = copy.deepcopy(value)
    for key, value in override.items():
        current = result.get(key)
        if key in result and isinstance(current, dict) and isinstance(value, dict):
            result[key] = deep_merge(current, value, list_strategy=list_strategy)
        elif key in result and isinstance(current, list) and isinstance(value, list):
            result[key] = _merge_lists(current, value, list_strategy)
        else:
            result[key] = copy.deepcopy(value)
    return result
