from collections.abc import Callable
from typing import Any


def map_csv_row(
    row: dict[str, str],
    schema: dict[str, Callable[[str], Any]],
) -> dict[str, Any]:
    """Convert CSV row to typed record using schema."""
    result = {}
    for key, converter in schema.items():
        if key in row:
            try:
                result[key] = converter(row[key])
            except (ValueError, TypeError):
                result[key] = None
        else:
            result[key] = None
    return result
