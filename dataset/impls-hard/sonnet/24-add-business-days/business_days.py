"""Add business days (Monday to Friday) to a date."""

from collections.abc import Collection
from datetime import date, timedelta


def add_business_days(start: date, days: int, holidays: Collection[date] = ()) -> date:
    """Return ``start`` moved by ``days`` business days.

    Saturdays, Sundays and any date in ``holidays`` are skipped. Negative
    ``days`` move backwards. Zero returns ``start`` unchanged.
    """
    step = 1 if days >= 0 else -1
    remaining = abs(days)
    current = start
    while remaining:
        current += timedelta(days=step)
        if current.weekday() < 5 and current not in holidays:
            remaining -= 1
    return current
