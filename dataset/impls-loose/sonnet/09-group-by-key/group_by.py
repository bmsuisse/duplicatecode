"""Group items into buckets by a computed key."""

from collections import defaultdict
from collections.abc import Callable, Hashable, Iterable


def group_by[T, K: Hashable](items: Iterable[T], key: Callable[[T], K]) -> dict[K, list[T]]:
    """Group ``items`` by ``key(item)``, preserving encounter order."""
    groups: defaultdict[K, list[T]] = defaultdict(list)
    for item in items:
        groups[key(item)].append(item)
    return dict(groups)
