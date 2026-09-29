from typing import Any, TypeVar

T = TypeVar("T")


def group_by_key[T](items: list[T], key_func) -> dict[Any, list[T]]:
    """Group items by computed key."""
    result: dict[Any, list[T]] = {}
    for item in items:
        key = key_func(item)
        if key not in result:
            result[key] = []
        result[key].append(item)
    return result
