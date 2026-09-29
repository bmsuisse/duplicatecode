"""Add business days to a date."""

from collections.abc import Collection
from datetime import date, timedelta


def add_business_days(start: date, days: int, holidays: Collection[date] = ()) -> date:
    """Move ``days`` Monday-Friday non-holiday days from ``start``."""
    step = timedelta(days=1 if days > 0 else -1)
    current = start
    remaining = abs(days)
    while remaining:
        current += step
        if current.weekday() < 5 and current not in holidays:
            remaining -= 1
    return current
