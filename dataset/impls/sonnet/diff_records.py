"""Diff two flat records."""

from collections.abc import Collection, Mapping
from typing import Any


def diff_records(
    old: Mapping[str, Any], new: Mapping[str, Any], *, ignore: Collection[str] = ()
) -> dict[str, dict[str, Any]]:
    """Report keys added, removed and changed between ``old`` and ``new``."""
    skipped = set(ignore)
    added: dict[str, Any] = {}
    removed: dict[str, Any] = {}
    changed: dict[str, Any] = {}
    for key, value in old.items():
        if key in skipped:
            continue
        if key not in new:
            removed[key] = value
        elif new[key] != value:
            changed[key] = {"old": value, "new": new[key]}
    for key, value in new.items():
        if key not in skipped and key not in old:
            added[key] = value
    return {"added": added, "removed": removed, "changed": changed}
