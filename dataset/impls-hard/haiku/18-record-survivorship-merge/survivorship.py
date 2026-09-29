from collections.abc import Callable
from typing import Any


def merge_records(
    records: list[dict[str, Any]],
    rules: dict[str, Callable[[list[Any]], Any]],
) -> dict[str, Any]:
    """Merge records using survivorship rules."""
    result = {}
    all_keys = set()

    for record in records:
        all_keys.update(record.keys())

    for key in all_keys:
        values = [r.get(key) for r in records if key in r]
        if key in rules:
            result[key] = rules[key](values)
        elif values:
            result[key] = values[0]

    return result
