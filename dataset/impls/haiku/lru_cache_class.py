from collections import OrderedDict


class LRUCache[K, V]:
    def __init__(self, capacity: int) -> None:
        if capacity < 1:
            raise ValueError("capacity must be >= 1")
        self.capacity = capacity
        self.cache: OrderedDict[K, V] = OrderedDict()

    def get(self, key: K, default: V | None = None) -> V | None:
        if key not in self.cache:
            return default
        # Mark as recently used
        self.cache.move_to_end(key)
        return self.cache[key]

    def put(self, key: K, value: V) -> None:
        if key in self.cache:
            # Update existing key and mark as recently used
            self.cache[key] = value
            self.cache.move_to_end(key)
        else:
            # Add new key
            self.cache[key] = value
            # Evict least recently used if over capacity
            if len(self.cache) > self.capacity:
                self.cache.popitem(last=False)

    def __len__(self) -> int:
        return len(self.cache)

    def __contains__(self, key: object) -> bool:
        return key in self.cache

    def keys(self) -> list[K]:
        return list(self.cache.keys())
