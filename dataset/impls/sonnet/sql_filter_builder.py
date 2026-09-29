"""Parameterised SQL WHERE clause builder."""

import re
from collections.abc import Mapping, Sequence
from typing import Any

_FIELD = re.compile(r"[A-Za-z_][A-Za-z0-9_.]*")
_COMPARISONS = {"eq": "=", "ne": "<>", "gt": ">", "gte": ">=", "lt": "<", "lte": "<="}


def build_where(
    filters: Sequence[Mapping[str, Any]], *, placeholder: str = "?"
) -> tuple[str, list[Any]]:
    """Build ``WHERE ...`` plus its ordered parameter list."""
    clauses: list[str] = []
    params: list[Any] = []

    def marks(count: int) -> str:
        return ", ".join([placeholder] * count)

    for spec in filters:
        field = spec["field"]
        op = spec["op"]
        value = spec.get("value")
        if not isinstance(field, str) or not _FIELD.fullmatch(field):
            raise ValueError(f"invalid field name: {field!r}")
        if op in ("eq", "ne") and value is None:
            clauses.append(f"{field} IS {'NOT ' if op == 'ne' else ''}NULL")
        elif op in _COMPARISONS:
            clauses.append(f"{field} {_COMPARISONS[op]} {placeholder}")
            params.append(value)
        elif op == "like":
            clauses.append(f"{field} LIKE {placeholder}")
            params.append(value)
        elif op in ("in", "not_in"):
            values = list(value or [])
            if not values:
                if op == "in":
                    clauses.append("1 = 0")
                continue
            keyword = "IN" if op == "in" else "NOT IN"
            clauses.append(f"{field} {keyword} ({marks(len(values))})")
            params.extend(values)
        elif op == "is_null":
            clauses.append(f"{field} IS {'NULL' if value else 'NOT NULL'}")
        elif op == "between":
            if not isinstance(value, (list, tuple)) or len(value) != 2:
                raise ValueError("between requires a list of two values")
            clauses.append(f"{field} BETWEEN {placeholder} AND {placeholder}")
            params.extend(value)
        else:
            raise ValueError(f"unknown operator: {op!r}")

    if not clauses:
        return "", []
    return "WHERE " + " AND ".join(clauses), params
