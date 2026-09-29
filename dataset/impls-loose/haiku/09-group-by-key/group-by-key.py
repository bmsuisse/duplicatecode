from collections.abc import Callable
from typing import Any


def group_by(items: list[Any], key_func: Callable[[Any], Any]) -> dict[Any, list[Any]]:
    """Group items by a computed key."""
    result: dict[Any, list[Any]] = {}
    for item in items:
        k = key_func(item)
        if k not in result:
            result[k] = []
        result[k].append(item)
    return result
