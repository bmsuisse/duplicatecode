from typing import Any, TypedDict


class PaginationInfo(TypedDict):
    """Pagination metadata."""

    current_page: int
    total_items: int
    total_pages: int
    page_size: int
    has_next: bool
    has_prev: bool


def paginate(
    items: list[Any],
    page: int,
    page_size: int,
) -> tuple[list[Any], PaginationInfo]:
    """Get a page of items with pagination info."""
    total = len(items)
    total_pages = (total + page_size - 1) // page_size
    start = (page - 1) * page_size
    end = start + page_size

    page_items = items[start:end]

    info: PaginationInfo = {
        "current_page": page,
        "total_items": total,
        "total_pages": total_pages,
        "page_size": page_size,
        "has_next": page < total_pages,
        "has_prev": page > 1,
    }

    return page_items, info
