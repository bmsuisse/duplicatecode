"""Group items by a computed key."""

from collections.abc import Callable, Hashable, Iterable


def group_by[T, K: Hashable](items: Iterable[T], key: Callable[[T], K]) -> dict[K, list[T]]:
    """Return a dict mapping each computed key to its items in input order."""
    groups: dict[K, list[T]] = {}
    for item in items:
        groups.setdefault(key(item), []).append(item)
    return groups
