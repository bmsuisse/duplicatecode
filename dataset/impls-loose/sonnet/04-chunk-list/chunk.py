"""Split sequences into fixed-size chunks."""

from collections.abc import Sequence


def chunk[T](items: Sequence[T], size: int) -> list[list[T]]:
    """Split ``items`` into consecutive lists of at most ``size`` elements."""
    if size < 1:
        raise ValueError("size must be a positive integer")
    return [list(items[i : i + size]) for i in range(0, len(items), size)]
