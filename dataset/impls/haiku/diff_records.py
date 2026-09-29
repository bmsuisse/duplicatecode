from collections import OrderedDict
from collections.abc import Collection, Mapping
from typing import Any


def diff_records(
    old: Mapping[str, Any],
    new: Mapping[str, Any],
    *,
    ignore: Collection[str] = (),
) -> dict[str, dict[str, Any]]:
    ignore_set = set(ignore)

    added: dict[str, Any] = OrderedDict()
    removed: dict[str, Any] = OrderedDict()
    changed: dict[str, dict[str, Any]] = OrderedDict()

    # Find removed and changed keys
    for key in old:
        if key in ignore_set:
            continue
        if key not in new:
            removed[key] = old[key]
        elif old[key] != new[key]:
            changed[key] = {"old": old[key], "new": new[key]}

    # Find added keys
    for key in new:
        if key in ignore_set:
            continue
        if key not in old:
            added[key] = new[key]

    return {"added": added, "removed": removed, "changed": changed}
