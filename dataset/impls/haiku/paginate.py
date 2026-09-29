import math
from collections.abc import Sequence
from dataclasses import dataclass


@dataclass(frozen=True)
class Page[T]:
    items: list[T]
    page: int
    page_size: int
    total_items: int
    total_pages: int
    has_next: bool
    has_prev: bool


def paginate[T](items: Sequence[T], page: int = 1, page_size: int = 20) -> Page[T]:
    if page_size < 1:
        raise ValueError("page_size must be >= 1")

    total_items = len(items)
    total_pages = math.ceil(total_items / page_size) if total_items > 0 else 0

    # Clamp page
    if page < 1:
        page = 1
    elif total_pages > 0 and page > total_pages:
        page = total_pages

    # Calculate slice
    start_idx = (page - 1) * page_size
    end_idx = start_idx + page_size
    page_items = list(items[start_idx:end_idx])

    has_prev = page > 1
    has_next = page < total_pages

    return Page(
        items=page_items,
        page=page,
        page_size=page_size,
        total_items=total_items,
        total_pages=total_pages,
        has_next=has_next,
        has_prev=has_prev,
    )
