from collections.abc import Callable
from typing import Any


def merge_records(
    records: list[dict[str, Any]],
    rules: dict[str, Callable[[list[Any]], Any]],
) -> dict[str, Any]:
    """Merge duplicate records with survivorship rules."""
    result = {}

    # Get all keys
    all_keys = set()
    for record in records:
        all_keys.update(record.keys())

    for key in all_keys:
        values = [r.get(key) for r in records if key in r]

        if key in rules:
            # Use rule to determine winner
            result[key] = rules[key](values)
        else:
            # Default: use first non-None value
            result[key] = next((v for v in values if v is not None), None)

    return result
