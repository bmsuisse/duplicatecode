from typing import Any


def build_where_clause(filters: dict[str, Any]) -> tuple[str, list[Any]]:
    """Build parameterised SQL WHERE clause from filter conditions."""
    conditions = []
    values = []

    for key, value in filters.items():
        if value is None:
            conditions.append(f"{key} IS NULL")
        else:
            conditions.append(f"{key} = %s")
            values.append(value)

    where_clause = " AND ".join(conditions) if conditions else "1=1"
    return where_clause, values
