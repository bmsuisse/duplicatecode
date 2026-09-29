"""A fixed-capacity least-recently-used cache."""

from collections import OrderedDict


class LRUCache[K, V]:
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
        """Return the value for ``key`` and mark it as recently used."""
        if key not in self._data:
            return default
        self._data.move_to_end(key)
        return self._data[key]

    def put(self, key: K, value: V) -> None:
        """Insert or update ``key``, evicting the oldest entry if necessary."""
        if key in self._data:
            self._data.move_to_end(key)
        self._data[key] = value
        if len(self._data) > self._capacity:
            self._data.popitem(last=False)

    def peek(self, key: K) -> V | None:
        """Return the value without changing the usage order."""
        return self._data.get(key)

    def discard(self, key: K) -> None:
        self._data.pop(key, None)

    def clear(self) -> None:
        self._data.clear()

    def __contains__(self, key: object) -> bool:
        return key in self._data

    def __len__(self) -> int:
        return len(self._data)
