from typing import Any


def diff_records(
    old: dict[str, Any],
    new: dict[str, Any],
    skip_keys: set[str] | None = None,
) -> dict[str, Any]:
    """Compare two records and report additions, removals, and changes."""
    if skip_keys is None:
        skip_keys = set()

    added = {}
    removed = {}
    changed = {}

    for key, value in new.items():
        if key in skip_keys:
            continue
        if key not in old:
            added[key] = value
        elif old[key] != value:
            changed[key] = {"old": old[key], "new": value}

    for key, value in old.items():
        if key in skip_keys:
            continue
        if key not in new:
            removed[key] = value

    return {"added": added, "removed": removed, "changed": changed}
