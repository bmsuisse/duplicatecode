from typing import Any


def chunk(items: list[Any], size: int) -> list[list[Any]]:
    """Split a list into chunks of given size."""
    return [items[i : i + size] for i in range(0, len(items), size)]
