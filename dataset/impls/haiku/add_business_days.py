from collections.abc import Collection
from datetime import date, timedelta


def add_business_days(start: date, days: int, holidays: Collection[date] = ()) -> date:
    holidays_set = set(holidays)

    def is_business_day(d: date) -> bool:
        # Monday is 0, Sunday is 6
        return d.weekday() < 5 and d not in holidays_set

    if days == 0:
        return start

    current = start
    remaining = abs(days)
    direction = 1 if days > 0 else -1

    while remaining > 0:
        current += timedelta(days=direction)
        if is_business_day(current):
            remaining -= 1

    return current
