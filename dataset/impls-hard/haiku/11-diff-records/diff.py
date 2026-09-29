from typing import Any


def diff_records(
    record1: dict[str, Any],
    record2: dict[str, Any],
    exclude_keys: set[str] | None = None,
) -> dict[str, dict[str, Any]]:
    """Compare two records identifying changes."""
    exclude = exclude_keys or set()
    result = {"added": {}, "removed": {}, "changed": {}}

    all_keys = set(record1.keys()) | set(record2.keys())

    for key in all_keys:
        if key in exclude:
            continue

        in_1 = key in record1
        in_2 = key in record2

        if not in_1 and in_2:
            result["added"][key] = record2[key]
        elif in_1 and not in_2:
            result["removed"][key] = record1[key]
        elif in_1 and in_2 and record1[key] != record2[key]:
            result["changed"][key] = {"old": record1[key], "new": record2[key]}

    return result
