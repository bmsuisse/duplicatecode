from collections import OrderedDict


class LRUCache[K, V]:
    """Fixed-capacity LRU cache that evicts least recently used items."""

    def __init__(self, capacity: int):
        if capacity <= 0:
            raise ValueError("Capacity must be positive")
        self.capacity = capacity
        self.cache: OrderedDict[K, V] = OrderedDict()

    def get(self, key: K) -> V:
        """Get a value from cache and mark as recently used."""
        if key not in self.cache:
            raise KeyError(f"Key not found: {key}")

        # Move to end to mark as recently used
        self.cache.move_to_end(key)
        return self.cache[key]

    def put(self, key: K, value: V) -> None:
        """Put a value in cache, evicting LRU item if necessary."""
        if key in self.cache:
            self.cache.move_to_end(key)

        self.cache[key] = value

        # Evict least recently used if over capacity
        if len(self.cache) > self.capacity:
            self.cache.popitem(last=False)

    def __len__(self) -> int:
        return len(self.cache)
