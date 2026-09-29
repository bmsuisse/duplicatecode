"""Short, localised labels for date ranges."""

import calendar
from datetime import date

_MONTHS = {
    "en": [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ],
    "de": [
        "Januar",
        "Februar",
        "März",
        "April",
        "Mai",
        "Juni",
        "Juli",
        "August",
        "September",
        "Oktober",
        "November",
        "Dezember",
    ],
}


def format_period_label(start: date, end: date | None = None, locale: str = "en") -> str:
    """Return e.g. ``March 2024``, ``March – May 2024`` or ``Dec 2023 – Feb 2024``-style labels.

    A range covering exactly one calendar month yields the single-month label;
    a range covering a whole year yields just the year.
    """
    end = end or start
    if end < start:
        raise ValueError("end must not be before start")
    months = _MONTHS.get(locale.split("-")[0].split("_")[0].lower())
    if months is None:
        raise ValueError(f"unsupported locale: {locale!r}")

    def label(d: date) -> str:
        return f"{months[d.month - 1]} {d.year}"

    whole_months = start.day == 1 and end.day == calendar.monthrange(end.year, end.month)[1]
    if whole_months and start.year == end.year and start.month == 1 and end.month == 12:
        return str(start.year)
    if (start.year, start.month) == (end.year, end.month):
        return label(start)
    if start.year == end.year:
        return f"{months[start.month - 1]} – {label(end)}"
    return f"{label(start)} – {label(end)}"
