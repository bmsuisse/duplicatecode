"""Convert CSV rows into typed records."""

import csv
from collections.abc import Callable, Iterable, Iterator, Mapping
from typing import Any, TextIO

Converter = Callable[[str], Any]


def parse_bool(text: str) -> bool:
    lowered = text.strip().lower()
    if lowered in {"1", "true", "yes", "y", "t"}:
        return True
    if lowered in {"0", "false", "no", "n", "f"}:
        return False
    raise ValueError(f"not a boolean: {text!r}")


def map_rows(
    rows: Iterable[Mapping[str, str]],
    converters: Mapping[str, Converter],
    *,
    empty_as_none: bool = True,
) -> Iterator[dict[str, Any]]:
    """Convert string columns using ``converters``; unmapped columns stay strings.

    Errors are re-raised with row number and column for easier debugging.
    """
    for number, row in enumerate(rows, start=1):
        record: dict[str, Any] = {}
        for column, text in row.items():
            if empty_as_none and (text is None or text.strip() == ""):
                record[column] = None
                continue
            convert = converters.get(column)
            if convert is None:
                record[column] = text
                continue
            try:
                record[column] = convert(text)
            except (ValueError, TypeError) as exc:
                raise ValueError(f"row {number}, column {column!r}: {exc}") from exc
        yield record


def read_csv(
    stream: TextIO, converters: Mapping[str, Converter], **kwargs: Any
) -> list[dict[str, Any]]:
    """Read CSV from ``stream`` (with header) into typed records."""
    return list(map_rows(csv.DictReader(stream), converters, **kwargs))
