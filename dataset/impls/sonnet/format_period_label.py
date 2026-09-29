"""Compact labels for inclusive date ranges."""

import calendar
from datetime import date
from typing import Literal

_MONTHS = {
    "en": ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
    "de": ["Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"],
}


def _last_day(year: int, month: int) -> int:
    return calendar.monthrange(year, month)[1]


def format_period(start: date, end: date, *, locale: Literal["en", "de"] = "en") -> str:
    """Return a short human label for the range ``start`` to ``end``."""
    if start > end:
        raise ValueError("start must not be after end")
    months = _MONTHS[locale]
    first_of_month = start.day == 1
    month_end = end.day == _last_day(end.year, end.month)
    same_month = (start.year, start.month) == (end.year, end.month)

    if first_of_month and month_end:
        if same_month:
            return f"{months[start.month - 1]} {start.year}"
        if start.year == end.year:
            if start.month == 1 and end.month == 12:
                return str(start.year)
            if (start.month - 1) % 3 == 0 and end.month - start.month == 2:
                return f"Q{(start.month - 1) // 3 + 1} {start.year}"
    if start == end:
        return f"{start.day} {months[start.month - 1]} {start.year}"
    if same_month:
        return f"{start.day}-{end.day} {months[start.month - 1]} {start.year}"
    if start.year == end.year:
        return (
            f"{start.day} {months[start.month - 1]} - {end.day} {months[end.month - 1]} {end.year}"
        )
    return (
        f"{start.day} {months[start.month - 1]} {start.year} - "
        f"{end.day} {months[end.month - 1]} {end.year}"
    )
