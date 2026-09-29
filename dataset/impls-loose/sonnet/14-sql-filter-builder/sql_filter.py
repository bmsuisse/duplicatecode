"""Build parameterised SQL WHERE clauses."""

import re
from collections.abc import Sequence
from dataclasses import dataclass
from typing import Any

_IDENTIFIER = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)?$")
_BINARY = {"=", "!=", "<>", "<", "<=", ">", ">=", "LIKE", "ILIKE"}


@dataclass(frozen=True)
class Condition:
    column: str
    op: str
    value: Any = None


def build_where(
    conditions: Sequence[Condition], *, joiner: str = "AND", placeholder: str = "%s"
) -> tuple[str, list[Any]]:
    """Return ``(sql, params)``; ``sql`` is empty when there are no conditions.

    Column names are validated as identifiers; values only ever appear in ``params``.
    Supported ops: comparison, LIKE/ILIKE, IN/NOT IN (sequence), BETWEEN (pair),
    IS NULL / IS NOT NULL.
    """
    if joiner.upper() not in {"AND", "OR"}:
        raise ValueError(f"invalid joiner: {joiner!r}")
    clauses: list[str] = []
    params: list[Any] = []
    for cond in conditions:
        if not _IDENTIFIER.match(cond.column):
            raise ValueError(f"invalid column name: {cond.column!r}")
        op = cond.op.strip().upper()
        if op in _BINARY:
            clauses.append(f"{cond.column} {op} {placeholder}")
            params.append(cond.value)
        elif op in {"IN", "NOT IN"}:
            values = list(cond.value)
            if not values:
                clauses.append("1 = 0" if op == "IN" else "1 = 1")
                continue
            clauses.append(f"{cond.column} {op} ({', '.join([placeholder] * len(values))})")
            params.extend(values)
        elif op == "BETWEEN":
            low, high = cond.value
            clauses.append(f"{cond.column} BETWEEN {placeholder} AND {placeholder}")
            params.extend([low, high])
        elif op in {"IS NULL", "IS NOT NULL"}:
            clauses.append(f"{cond.column} {op}")
        else:
            raise ValueError(f"unsupported operator: {cond.op!r}")
    if not clauses:
        return "", []
    return "WHERE " + f" {joiner.upper()} ".join(clauses), params
