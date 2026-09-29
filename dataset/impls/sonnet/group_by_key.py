"""Group or count items by a computed key."""

from collections.abc import Callable, Iterable


def group_by[T, K](items: Iterable[T], key: Callable[[T], K]) -> dict[K, list[T]]:
    """Map each key to the list of items producing it, in encounter order."""
    groups: dict[K, list[T]] = {}
    for item in items:
        groups.setdefault(key(item), []).append(item)
    return groups


def count_by[T, K](items: Iterable[T], key: Callable[[T], K]) -> dict[K, int]:
    """Map each key to how many items produced it."""
    counts: dict[K, int] = {}
    for item in items:
        k = key(item)
        counts[k] = counts.get(k, 0) + 1
    return counts
