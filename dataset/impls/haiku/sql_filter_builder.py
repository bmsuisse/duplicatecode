import re
from collections.abc import Mapping, Sequence
from typing import Any


def build_where(
    filters: Sequence[Mapping[str, Any]], *, placeholder: str = "?"
) -> tuple[str, list[Any]]:
    if not filters:
        return ("", [])

    op_map = {
        "eq": "=",
        "ne": "<>",
        "gt": ">",
        "gte": ">=",
        "lt": "<",
        "lte": "<=",
        "like": "LIKE",
    }

    clauses = []
    params: list[Any] = []

    for filter_item in filters:
        field = filter_item["field"]
        op = filter_item["op"]
        value = filter_item["value"]

        # Validate field name
        if not re.match(r"^[A-Za-z_][A-Za-z0-9_.]*$", field):
            raise ValueError(f"Invalid field name: {field}")

        # Validate operator
        if op not in op_map and op not in ["in", "not_in", "is_null", "between"]:
            raise ValueError(f"Unknown operator: {op}")

        if op == "eq":
            if value is None:
                clauses.append(f"{field} IS NULL")
            else:
                clauses.append(f"{field} = {placeholder}")
                params.append(value)
        elif op == "ne":
            if value is None:
                clauses.append(f"{field} IS NOT NULL")
            else:
                clauses.append(f"{field} <> {placeholder}")
                params.append(value)
        elif op == "is_null":
            if value:
                clauses.append(f"{field} IS NULL")
            else:
                clauses.append(f"{field} IS NOT NULL")
        elif op == "in":
            if not value:
                clauses.append("1 = 0")
            else:
                placeholders = ",".join([placeholder] * len(value))
                clauses.append(f"{field} IN ({placeholders})")
                params.extend(value)
        elif op == "not_in":
            if value:
                placeholders = ",".join([placeholder] * len(value))
                clauses.append(f"{field} NOT IN ({placeholders})")
                params.extend(value)
        elif op == "between":
            if len(value) != 2:
                raise ValueError("between requires 2-item list")
            clauses.append(f"{field} BETWEEN {placeholder} AND {placeholder}")
            params.extend(value)
        else:
            # eq, ne, gt, gte, lt, lte, like
            clauses.append(f"{field} {op_map[op]} {placeholder}")
            params.append(value)

    where_clause = "WHERE " + " AND ".join(clauses)
    return (where_clause, params)
