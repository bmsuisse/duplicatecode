from typing import Any


class LRUCache:
    """Fixed-capacity LRU cache that evicts least-recently-used items."""

    def __init__(self, capacity: int):
        if capacity <= 0:
            raise ValueError("Capacity must be positive")
        self.capacity = capacity
        self.cache: dict[Any, Any] = {}
        self.access_order: list[Any] = []

    def get(self, key: Any) -> Any:
        """Get a value and mark it as recently used."""
        if key not in self.cache:
            raise KeyError(key)
        self.access_order.remove(key)
        self.access_order.append(key)
        return self.cache[key]

    def put(self, key: Any, value: Any) -> None:
        """Store a value. Evict LRU item if at capacity."""
        if key in self.cache:
            self.access_order.remove(key)
        elif len(self.cache) >= self.capacity:
            lru_key = self.access_order.pop(0)
            del self.cache[lru_key]

        self.cache[key] = value
        self.access_order.append(key)
