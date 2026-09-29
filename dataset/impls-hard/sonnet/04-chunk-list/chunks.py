"""Split a sequence into fixed-size chunks."""

from collections.abc import Sequence


def chunk[T](items: Sequence[T], size: int) -> list[list[T]]:
    """Split ``items`` into consecutive lists of ``size`` elements.

    The order is preserved and the last chunk may be shorter.
    """
    if size < 1:
        raise ValueError("size must be at least 1")
    return [list(items[start : start + size]) for start in range(0, len(items), size)]
