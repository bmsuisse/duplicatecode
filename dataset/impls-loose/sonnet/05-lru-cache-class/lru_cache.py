"""A small fixed-capacity LRU cache."""

from collections import OrderedDict
from collections.abc import Hashable


class LRUCache[K: Hashable, V]:
    """Cache that evicts the least recently used entry when full."""

    def __init__(self, capacity: int) -> None:
        if capacity < 1:
            raise ValueError("capacity must be at least 1")
        self._capacity = capacity
        self._data: OrderedDict[K, V] = OrderedDict()

    @property
    def capacity(self) -> int:
        return self._capacity

    def get(self, key: K, default: V | None = None) -> V | None:
        """Return the value for ``key`` (marking it recently used) or ``default``."""
        if key not in self._data:
            return default
        self._data.move_to_end(key)
        return self._data[key]

    def put(self, key: K, value: V) -> None:
        """Insert or update ``key``, evicting the oldest entry if over capacity."""
        self._data[key] = value
        self._data.move_to_end(key)
        if len(self._data) > self._capacity:
            self._data.popitem(last=False)

    def pop(self, key: K, default: V | None = None) -> V | None:
        return self._data.pop(key, default)

    def clear(self) -> None:
        self._data.clear()

    def __contains__(self, key: object) -> bool:
        return key in self._data

    def __len__(self) -> int:
        return len(self._data)
