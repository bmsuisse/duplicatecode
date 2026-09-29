"""Compare two flat records."""

from collections.abc import Collection, Mapping
from dataclasses import dataclass, field
from typing import Any


@dataclass(frozen=True)
class RecordDiff:
    added: dict[str, Any] = field(default_factory=dict)
    removed: dict[str, Any] = field(default_factory=dict)
    changed: dict[str, tuple[Any, Any]] = field(default_factory=dict)

    def __bool__(self) -> bool:
        return bool(self.added or self.removed or self.changed)


def diff_records(
    old: Mapping[str, Any],
    new: Mapping[str, Any],
    ignore: Collection[str] = (),
) -> RecordDiff:
    """Return the fields added in ``new``, removed from ``old`` and changed.

    ``changed`` maps a field name to ``(old_value, new_value)``. Fields listed in
    ``ignore`` are excluded from the comparison.
    """
    skip = set(ignore)
    old_keys = {k for k in old if k not in skip}
    new_keys = {k for k in new if k not in skip}
    return RecordDiff(
        added={k: new[k] for k in new if k in new_keys - old_keys},
        removed={k: old[k] for k in old if k in old_keys - new_keys},
        changed={k: (old[k], new[k]) for k in old if k in old_keys & new_keys and old[k] != new[k]},
    )
