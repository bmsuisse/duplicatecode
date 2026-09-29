"""Build a parameterised SQL WHERE clause from filter conditions."""

import re
from collections.abc import Sequence
from dataclasses import dataclass
from typing import Any

_IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)?")
_COMPARISONS = {"=", "!=", "<>", "<", "<=", ">", ">=", "LIKE", "NOT LIKE"}


@dataclass(frozen=True)
class Condition:
    column: str
    operator: str
    value: Any = None


@dataclass(frozen=True)
class Group:
    """A set of conditions/groups combined with ``AND`` or ``OR``."""

    items: Sequence["Condition | Group"]
    logic: str = "AND"


def build_where(node: Condition | Group, placeholder: str = "?") -> tuple[str, list[Any]]:
    """Return ``(sql, params)`` where ``sql`` has no user values, only placeholders.

    Column names are validated against a strict identifier pattern and operators
    against a whitelist. Supported operators: comparisons, ``LIKE``, ``IN``,
    ``NOT IN``, ``BETWEEN``, ``IS NULL`` and ``IS NOT NULL``. An empty group
    yields ``("", [])``.
    """
    params: list[Any] = []
    sql = _render(node, placeholder, params)
    return sql, params


def _render(node: Condition | Group, ph: str, params: list[Any]) -> str:
    if isinstance(node, Group):
        logic = node.logic.upper()
        if logic not in ("AND", "OR"):
            raise ValueError(f"invalid logic: {node.logic!r}")
        parts = [p for p in (_render(item, ph, params) for item in node.items) if p]
        if not parts:
            return ""
        if len(parts) == 1:
            return parts[0]
        return "(" + f" {logic} ".join(parts) + ")"

    if not _IDENTIFIER.fullmatch(node.column):
        raise ValueError(f"invalid column name: {node.column!r}")
    op = " ".join(node.operator.upper().split())
    column = node.column
    if op in ("IS NULL", "IS NOT NULL"):
        return f"{column} {op}"
    if op in ("IN", "NOT IN"):
        values = list(node.value)
        if not values:
            # An empty IN list never matches; an empty NOT IN list always does.
            return "1 = 0" if op == "IN" else "1 = 1"
        params.extend(values)
        return f"{column} {op} ({', '.join([ph] * len(values))})"
    if op == "BETWEEN":
        low, high = node.value
        params.extend([low, high])
        return f"{column} BETWEEN {ph} AND {ph}"
    if op in _COMPARISONS:
        params.append(node.value)
        return f"{column} {op} {ph}"
    raise ValueError(f"unsupported operator: {node.operator!r}")
