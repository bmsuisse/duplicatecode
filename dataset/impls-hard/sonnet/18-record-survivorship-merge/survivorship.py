"""Merge duplicate records into one golden record using per-field rules."""

from collections.abc import Callable, Mapping, Sequence
from typing import Any

type Record = Mapping[str, Any]
type Rule = Callable[[str, Sequence[Record]], Any]


def _is_blank(value: Any) -> bool:
    return value is None or (isinstance(value, str) and not value.strip())


def most_recent(timestamp_field: str) -> Rule:
    """Take the non-blank value from the record with the newest timestamp."""

    def rule(field: str, records: Sequence[Record]) -> Any:
        ordered = sorted(
            (r for r in records if timestamp_field in r and r[timestamp_field] is not None),
            key=lambda r: r[timestamp_field],
            reverse=True,
        )
        for record in ordered:
            if not _is_blank(record.get(field)):
                return record[field]
        return first_non_blank(field, records)

    return rule


def first_non_blank(field: str, records: Sequence[Record]) -> Any:
    """Take the first non-blank value in input order."""
    for record in records:
        if not _is_blank(record.get(field)):
            return record[field]
    return None


def longest(field: str, records: Sequence[Record]) -> Any:
    """Take the most complete (longest string form) value."""
    values = [r[field] for r in records if not _is_blank(r.get(field))]
    return max(values, key=lambda v: len(str(v)), default=None)


def most_common(field: str, records: Sequence[Record]) -> Any:
    """Take the most frequent non-blank value; ties go to the earliest."""
    values = [r[field] for r in records if not _is_blank(r.get(field))]
    counts: dict[Any, int] = {}
    for value in values:
        key = repr(value)
        counts[key] = counts.get(key, 0) + 1
    if not values:
        return None
    return max(values, key=lambda v: counts[repr(v)])


def merge_records(
    records: Sequence[Record],
    rules: Mapping[str, Rule] | None = None,
    default_rule: Rule = first_non_blank,
) -> dict[str, Any]:
    """Build one record from ``records``.

    Every field found in any record is resolved with the rule registered for it
    in ``rules`` or, failing that, ``default_rule``.
    """
    if not records:
        raise ValueError("at least one record is required")
    rules = rules or {}
    fields: dict[str, None] = {}
    for record in records:
        fields.update(dict.fromkeys(record))
    return {name: rules.get(name, default_rule)(name, records) for name in fields}
