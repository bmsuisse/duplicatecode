"""Pagination helper."""

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
    """Return one 1-based page of ``items``, clamping ``page`` into range."""
    if page_size < 1:
        raise ValueError("page_size must be at least 1")
    total = len(items)
    total_pages = -(-total // page_size)
    current = max(1, min(page, total_pages)) if total_pages else 1
    start = (current - 1) * page_size
    return Page(
        items=list(items[start : start + page_size]),
        page=current,
        page_size=page_size,
        total_items=total,
        total_pages=total_pages,
        has_next=current < total_pages,
        has_prev=current > 1,
    )
