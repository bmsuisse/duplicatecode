# Split a sequence into chunks

- **Language:** python
- **Target file:** `chunk_list.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def chunk(items: Sequence[T], size: int, *, drop_remainder: bool = False) -> list[list[T]]
```

## Behavior

Split `items` into consecutive lists of length `size`. The last chunk may be shorter unless `drop_remainder` is True, in which case an incomplete last chunk is discarded.

## Edge cases

- size <= 0 -> ValueError.
- Empty input -> [].
- Works with strings (chunks are lists of characters) and tuples.
- size larger than len -> one chunk (or [] if drop_remainder).

## Examples

- chunk([1,2,3,4,5], 2) -> [[1,2],[3,4],[5]]
- chunk([1,2,3,4,5], 2, drop_remainder=True) -> [[1,2],[3,4]]
- chunk([], 3) -> []
- chunk("abc", 5) -> [["a","b","c"]]
- chunk([1], 0) -> ValueError
