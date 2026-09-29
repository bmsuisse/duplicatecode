from collections.abc import Sequence


def chunk[T](items: Sequence[T], size: int, *, drop_remainder: bool = False) -> list[list[T]]:
    if size <= 0:
        raise ValueError("size must be > 0")

    result = []
    for i in range(0, len(items), size):
        chunk_items = list(items[i : i + size])
        if len(chunk_items) == size or not drop_remainder:
            result.append(chunk_items)

    return result
