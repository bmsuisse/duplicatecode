# Group items by computed key

- **Language:** python
- **Target file:** `group_by_key.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def group_by(items: Iterable[T], key: Callable[[T], K]) -> dict[K, list[T]]
def count_by(items: Iterable[T], key: Callable[[T], K]) -> dict[K, int]
```

## Behavior

`group_by` returns a dict mapping each key to the list of items with that key, preserving encounter order both for keys (first seen) and for items within groups. `count_by` returns key -> number of items, same key ordering.

## Edge cases

- Empty iterable -> {}.
- Iterables may be one-shot generators (consume once).
- Keys must be hashable; unhashable raises TypeError naturally.

## Examples

- group_by(["apple","avocado","banana"], lambda s: s[0]) -> {"a":["apple","avocado"],"b":["banana"]}
- group_by(range(6), lambda n: n % 3) -> {0:[0,3],1:[1,4],2:[2,5]}
- count_by("mississippi", lambda c: c) -> {"m":1,"i":4,"s":4,"p":2}
- group_by([], str) -> {}
