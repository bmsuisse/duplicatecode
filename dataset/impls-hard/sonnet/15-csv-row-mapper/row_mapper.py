"""Convert rows of text values into typed records."""

from collections.abc import Callable, Mapping
from dataclasses import dataclass
from datetime import UTC, datetime
from decimal import Decimal, InvalidOperation
from typing import Any

_TRUE = {"true", "1", "yes", "y", "t", "ja"}
_FALSE = {"false", "0", "no", "n", "f", "nein"}


def parse_bool(text: str) -> bool:
    lowered = text.strip().lower()
    if lowered in _TRUE:
        return True
    if lowered in _FALSE:
        return False
    raise ValueError(f"not a boolean: {text!r}")


def parse_decimal(text: str) -> Decimal:
    try:
        return Decimal(text.strip())
    except InvalidOperation as exc:
        raise ValueError(f"not a decimal: {text!r}") from exc


@dataclass(frozen=True)
class Column:
    """Description of one column: its source name, target type and options."""

    name: str
    parse: Callable[[str], Any] = str
    required: bool = False
    default: Any = None
    target: str | None = None


def int_column(name: str, **kwargs: Any) -> Column:
    return Column(name, lambda s: int(s.strip()), **kwargs)


def decimal_column(name: str, **kwargs: Any) -> Column:
    return Column(name, parse_decimal, **kwargs)


def bool_column(name: str, **kwargs: Any) -> Column:
    return Column(name, parse_bool, **kwargs)


def date_column(name: str, fmt: str = "%Y-%m-%d", **kwargs: Any) -> Column:
    return Column(
        name, lambda s: datetime.strptime(s.strip(), fmt).replace(tzinfo=UTC).date(), **kwargs
    )


def map_row(row: Mapping[str, str], columns: list[Column]) -> dict[str, Any]:
    """Convert one text row to a typed record following ``columns``.

    Empty or missing cells yield the column default (or raise if required).
    Conversion errors are re-raised as ``ValueError`` naming the column.
    """
    record: dict[str, Any] = {}
    for col in columns:
        raw = row.get(col.name)
        key = col.target or col.name
        if raw is None or raw.strip() == "":
            if col.required:
                raise ValueError(f"missing required value for column {col.name!r}")
            record[key] = col.default
            continue
        try:
            record[key] = col.parse(raw)
        except ValueError as exc:
            raise ValueError(f"column {col.name!r}: {exc}") from exc
    return record


def map_rows(rows: list[Mapping[str, str]], columns: list[Column]) -> list[dict[str, Any]]:
    return [map_row(row, columns) for row in rows]


__all__ = [
    "Column",
    "bool_column",
    "date_column",
    "decimal_column",
    "int_column",
    "map_row",
    "map_rows",
    "parse_bool",
    "parse_decimal",
]
