from typing import Any


def build_sql_where(filters: dict[str, Any]) -> tuple[str, list[Any]]:
    """Build parameterized SQL WHERE clause."""
    conditions = []
    params = []

    for key, value in filters.items():
        conditions.append(f"{key} = %s")
        params.append(value)

    where_clause = " AND ".join(conditions) if conditions else "1=1"
    return where_clause, params
