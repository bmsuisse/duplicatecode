"""Slice a list into pages with navigation metadata."""

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

    @property
    def has_previous(self) -> bool:
        return self.page > 1

    @property
    def has_next(self) -> bool:
        return self.page < self.total_pages


def paginate[T](items: Sequence[T], page: int = 1, page_size: int = 20) -> Page[T]:
    """Return the requested 1-based ``page`` of ``items``.

    A page beyond the last one is clamped to the last page; an empty input has
    zero pages and yields an empty first page.
    """
    if page < 1:
        raise ValueError("page must be at least 1")
    if page_size < 1:
        raise ValueError("page_size must be at least 1")
    total = len(items)
    total_pages = math.ceil(total / page_size)
    current = min(page, max(total_pages, 1))
    start = (current - 1) * page_size
    return Page(list(items[start : start + page_size]), current, page_size, total, total_pages)
