"""Map CSV rows to typed records."""

from collections.abc import Callable, Iterable, Mapping, Sequence
from typing import Any


def map_rows(
    rows: Iterable[Sequence[str]],
    mapping: Mapping[str, tuple[str, Callable[[str], Any]]],
    *,
    has_header: bool = True,
) -> tuple[list[dict[str, Any]], list[str]]:
    """Convert rows into records; returns (records, error messages)."""
    wanted = {name.strip().lower(): target for name, target in mapping.items()}
    iterator = iter(rows)
    columns: dict[str, int] = {}
    if has_header:
        header = next(iterator, [])
        for index, name in enumerate(header):
            columns[name.strip().lower()] = index
        missing = [name for name in wanted if name not in columns]
        if missing:
            raise ValueError(f"missing columns in header: {', '.join(missing)}")
    else:
        columns = {name: int(name) for name in wanted}

    records: list[dict[str, Any]] = []
    errors: list[str] = []
    for number, row in enumerate(iterator, start=1):
        if all(not cell.strip() for cell in row):
            continue
        record: dict[str, Any] = {}
        try:
            for name, (out_key, convert) in wanted.items():
                index = columns[name]
                cell = row[index].strip() if index < len(row) else ""
                try:
                    record[out_key] = convert(cell) if cell else None
                except ValueError as exc:
                    raise _RowError(f"row {number}: {out_key}: {exc}") from exc
        except _RowError as exc:
            errors.append(str(exc))
            continue
        records.append(record)
    return records, errors


class _RowError(Exception):
    """Internal signal used to skip a row after a conversion failure."""
