"""Compare two flat records."""

from collections.abc import Collection, Mapping
from dataclasses import dataclass, field
from typing import Any


@dataclass(frozen=True)
class RecordDiff:
    added: dict[str, Any] = field(default_factory=dict)
    removed: dict[str, Any] = field(default_factory=dict)
    changed: dict[str, tuple[Any, Any]] = field(default_factory=dict)  # key -> (old, new)

    def __bool__(self) -> bool:
        return bool(self.added or self.removed or self.changed)


def diff_records(
    old: Mapping[str, Any], new: Mapping[str, Any], ignore: Collection[str] = ()
) -> RecordDiff:
    """Report keys added to, removed from, or changed between ``old`` and ``new``."""
    skip = set(ignore)
    added = {k: v for k, v in new.items() if k not in old and k not in skip}
    removed = {k: v for k, v in old.items() if k not in new and k not in skip}
    changed = {
        k: (old[k], new[k]) for k in old.keys() & new.keys() if k not in skip and old[k] != new[k]
    }
    return RecordDiff(added, removed, changed)
