"""Short, localised labels for reporting periods."""

import calendar
from datetime import date

_MONTHS = {
    "en": [
        "January", "February", "March", "April", "May", "June", "July",
        "August", "September", "October", "November", "December",
    ],
    "de": [
        "Januar", "Februar", "März", "April", "Mai", "Juni", "Juli",
        "August", "September", "Oktober", "November", "Dezember",
    ],
}  # fmt: skip
_SHORT_MONTHS = {
    "en": ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
    "de": ["Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"],
}


def _short_date(day: date, lang: str) -> str:
    month = _SHORT_MONTHS[lang][day.month - 1]
    if lang == "de":
        return f"{day.day}. {month} {day.year}"
    return f"{month} {day.day}, {day.year}"


def format_period_label(start: date, end: date, lang: str = "en") -> str:
    """Return ``"March 2024"`` for a full calendar month, else a range.

    Ranges look like ``"Mar 3, 2024 - Apr 9, 2024"`` (en) or
    ``"3. Mär 2024 - 9. Apr 2024"`` (de). A single day is shown as one date.
    """
    if lang not in _MONTHS:
        raise ValueError(f"unsupported language: {lang!r}")
    if end < start:
        raise ValueError("end must not be before start")
    last_day = calendar.monthrange(start.year, start.month)[1]
    if start.day == 1 and end == date(start.year, start.month, last_day):
        return f"{_MONTHS[lang][start.month - 1]} {start.year}"
    if start == end:
        return _short_date(start, lang)
    return f"{_short_date(start, lang)} - {_short_date(end, lang)}"
