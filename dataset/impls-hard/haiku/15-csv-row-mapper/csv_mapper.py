from typing import Any


def map_csv_row(row: list[str], schema: dict[str, type]) -> dict[str, Any]:
    """Map CSV row to typed record with type conversion."""
    result = {}
    for i, (key, type_hint) in enumerate(schema.items()):
        if i >= len(row):
            break
        try:
            result[key] = type_hint(row[i])
        except (ValueError, TypeError):
            result[key] = None
    return result
