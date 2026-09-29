"""Merge duplicate records into a golden record using per-field rules."""

from collections.abc import Callable, Mapping, Sequence
from typing import Any

Rule = Callable[[Sequence[Any]], Any]


def _is_empty(value: Any) -> bool:
    return value is None or (isinstance(value, str) and not value.strip())


def first_non_empty(values: Sequence[Any]) -> Any:
    return next((v for v in values if not _is_empty(v)), None)


def last_non_empty(values: Sequence[Any]) -> Any:
    return next((v for v in reversed(values) if not _is_empty(v)), None)


def longest(values: Sequence[Any]) -> Any:
    candidates = [v for v in values if not _is_empty(v)]
    return max(candidates, key=lambda v: len(str(v)), default=None)


def most_common(values: Sequence[Any]) -> Any:
    candidates = [v for v in values if not _is_empty(v)]
    # max() keeps the first of equal counts, so ties resolve to the earliest record.
    return max(candidates, key=candidates.count, default=None)


def maximum(values: Sequence[Any]) -> Any:
    return max((v for v in values if not _is_empty(v)), default=None)


def minimum(values: Sequence[Any]) -> Any:
    return min((v for v in values if not _is_empty(v)), default=None)


def merge_records(
    records: Sequence[Mapping[str, Any]],
    rules: Mapping[str, Rule] | None = None,
    default_rule: Rule = first_non_empty,
) -> dict[str, Any]:
    """Merge ``records`` (ordered by priority, best first) into one record.

    ``rules`` maps a field name to a function choosing the surviving value from
    the list of that field's values across all records; other fields use ``default_rule``.
    """
    rules = rules or {}
    fields: dict[str, None] = {}
    for record in records:
        fields.update(dict.fromkeys(record))
    return {name: rules.get(name, default_rule)([r.get(name) for r in records]) for name in fields}
