from collections.abc import Iterator
from typing import TypeVar

T = TypeVar("T")


def chunk_list[T](items: list[T], chunk_size: int) -> Iterator[list[T]]:
    """Split a list into chunks of specified size."""
    if chunk_size <= 0:
        raise ValueError("Chunk size must be positive")

    for i in range(0, len(items), chunk_size):
        yield items[i : i + chunk_size]
