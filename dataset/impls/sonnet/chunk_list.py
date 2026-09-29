"""Split a sequence into fixed-size chunks."""

from collections.abc import Sequence


def chunk[T](items: Sequence[T], size: int, *, drop_remainder: bool = False) -> list[list[T]]:
    """Split ``items`` into lists of ``size`` elements."""
    if size <= 0:
        raise ValueError("size must be positive")
    chunks = [list(items[i : i + size]) for i in range(0, len(items), size)]
    if drop_remainder and chunks and len(chunks[-1]) < size:
        chunks.pop()
    return chunks
