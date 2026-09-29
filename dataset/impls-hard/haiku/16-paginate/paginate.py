from typing import Any, TypeVar

T = TypeVar("T")


def paginate[T](items: list[T], page: int, page_size: int) -> dict[str, Any]:
    """Paginate list returning slice and metadata."""
    total = len(items)
    total_pages = (total + page_size - 1) // page_size

    start = (page - 1) * page_size
    end = start + page_size

    return {
        "items": items[start:end],
        "page": page,
        "page_size": page_size,
        "total": total,
        "total_pages": total_pages,
        "has_next": page < total_pages,
        "has_previous": page > 1,
    }
