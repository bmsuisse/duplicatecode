"""Master-data record merge with survivorship rules."""

from collections.abc import Mapping, Sequence
from typing import Any, Literal

Rule = Literal["most_recent", "most_complete", "first_non_null", "longest"]


def _empty(value: Any) -> bool:
    return value is None or value == ""


def merge_records(
    records: Sequence[Mapping[str, Any]],
    rules: Mapping[str, Rule],
    *,
    updated_key: str = "updated_at",
) -> dict[str, Any]:
    """Merge duplicate records into a single golden record."""
    if not records:
        raise ValueError("records must not be empty")

    fields: list[str] = []
    for record in records:
        for key in record:
            if key != updated_key and key not in fields:
                fields.append(key)

    def stamp(record: Mapping[str, Any]) -> Any:
        return record.get(updated_key) or ""

    by_recency = sorted(records, key=stamp, reverse=True)

    def first_non_null(field: str) -> Any:
        return next((r[field] for r in records if not _empty(r.get(field))), None)

    def most_recent(field: str) -> Any:
        return next((r[field] for r in by_recency if not _empty(r.get(field))), None)

    def most_complete(field: str) -> Any:
        best = max(
            by_recency, key=lambda r: sum(not _empty(v) for k, v in r.items() if k != updated_key)
        )
        value = best.get(field)
        return first_non_null(field) if _empty(value) else value

    def longest(field: str) -> Any:
        values = [r[field] for r in records if not _empty(r.get(field))]
        return max(values, key=lambda v: len(str(v)), default=None)

    strategies = {
        "most_recent": most_recent,
        "most_complete": most_complete,
        "first_non_null": first_non_null,
        "longest": longest,
    }
    merged = {field: strategies[rules.get(field, "first_non_null")](field) for field in fields}
    stamps = [r[updated_key] for r in records if not _empty(r.get(updated_key))]
    merged[updated_key] = max(stamps) if stamps else None
    return merged
