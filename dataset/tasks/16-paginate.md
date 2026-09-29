# Pagination helper

- **Language:** python
- **Target file:** `paginate.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
@dataclass(frozen=True)
class Page(Generic[T]):
    items: list[T]; page: int; page_size: int; total_items: int; total_pages: int; has_next: bool; has_prev: bool

def paginate(items: Sequence[T], page: int = 1, page_size: int = 20) -> Page[T]
```

## Behavior

1-based page. total_pages = ceil(total_items / page_size), which is 0 for empty input. `page` beyond total_pages is clamped to the last page (or 1 when total_pages is 0); page < 1 is clamped to 1. has_prev = page > 1; has_next = page < total_pages.

## Edge cases

- page_size < 1 -> ValueError.
- Empty input -> items [], page 1, total_pages 0, has_next False.
- Returned `page` reflects the clamped value.

## Examples

- 45 items, page=2, page_size=20 -> items[20:40], total_pages=3, has_next True, has_prev True
- 45 items, page=9, page_size=20 -> page=3, 5 items
- [] -> Page([],1,20,0,0,False,False)
- 10 items, page=0, page_size=5 -> page=1, items[0:5]
