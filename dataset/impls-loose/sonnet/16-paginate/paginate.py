"""Pagination helper."""

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

    @property
    def previous_page(self) -> int | None:
        return self.page - 1 if self.has_previous else None

    @property
    def next_page(self) -> int | None:
        return self.page + 1 if self.has_next else None


def paginate[T](items: Sequence[T], page: int = 1, page_size: int = 20) -> Page[T]:
    """Return the 1-based ``page`` of ``items``; out-of-range pages are clamped."""
    if page_size < 1:
        raise ValueError("page_size must be positive")
    total = len(items)
    total_pages = max(1, math.ceil(total / page_size))
    page = min(max(page, 1), total_pages)
    start = (page - 1) * page_size
    return Page(list(items[start : start + page_size]), page, page_size, total, total_pages)
