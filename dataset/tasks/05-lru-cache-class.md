# Fixed-capacity LRU cache

- **Language:** python
- **Target file:** `lru_cache_class.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
class LRUCache(Generic[K, V]):
    def __init__(self, capacity: int) -> None
    def get(self, key: K, default: V | None = None) -> V | None
    def put(self, key: K, value: V) -> None
    def __len__(self) -> int
    def __contains__(self, key: object) -> bool
    def keys(self) -> list[K]  # least recently used first
```

## Behavior

Least-recently-used cache. `get` on a hit marks the key most recently used. `put` inserts/updates (updating also marks most recent) and evicts the least recently used entry when size would exceed capacity. `__contains__` and `keys` do NOT change recency. Implement without functools.lru_cache.

## Edge cases

- capacity < 1 -> ValueError.
- get of missing key returns `default`.
- Storing None as a value is allowed and distinguishable via `in`.

## Examples

- capacity 2: put a, put b, put c -> keys() == ["b","c"]
- capacity 2: put a, put b, get a, put c -> keys() == ["a","c"]
- put a=1, put a=2 -> len == 1, get a == 2
- capacity 0 -> ValueError
