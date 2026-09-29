"""Recursive dictionary merge that never mutates its inputs."""

import copy
from collections.abc import Mapping
from typing import Any


def deep_merge(defaults: Mapping[str, Any], overrides: Mapping[str, Any]) -> dict[str, Any]:
    """Return a new dict where ``overrides`` win over ``defaults``.

    Nested mappings are merged recursively; all other values (including lists)
    from ``overrides`` replace the default value. Inputs are left untouched.
    """
    result: dict[str, Any] = {key: copy.deepcopy(value) for key, value in defaults.items()}
    for key, value in overrides.items():
        current = result.get(key)
        if isinstance(current, Mapping) and isinstance(value, Mapping):
            result[key] = deep_merge(current, value)
        else:
            result[key] = copy.deepcopy(value)
    return result
