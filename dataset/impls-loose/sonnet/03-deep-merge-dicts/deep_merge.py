"""Recursive dictionary merge."""

import copy
from collections.abc import Mapping
from typing import Any


def deep_merge(base: Mapping[str, Any], override: Mapping[str, Any]) -> dict[str, Any]:
    """Return a new dict with ``override`` merged recursively into ``base``.

    Nested mappings are combined; any other value from ``override`` wins.
    Neither input is modified and the result shares no mutable state with them.
    """
    result: dict[str, Any] = {k: copy.deepcopy(v) for k, v in base.items()}
    for key, value in override.items():
        existing = result.get(key)
        if isinstance(existing, Mapping) and isinstance(value, Mapping):
            result[key] = deep_merge(existing, value)
        else:
            result[key] = copy.deepcopy(value)
    return result
