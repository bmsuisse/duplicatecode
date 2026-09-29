from datetime import date, timedelta
from typing import Literal


def format_period(start: date, end: date, *, locale: Literal["en", "de"] = "en") -> str:
    if start > end:
        raise ValueError("start must be <= end")

    month_names_en = [
        "Jan",
        "Feb",
        "Mar",
        "Apr",
        "May",
        "Jun",
        "Jul",
        "Aug",
        "Sep",
        "Oct",
        "Nov",
        "Dec",
    ]
    month_names_de = [
        "Jan",
        "Feb",
        "Mär",
        "Apr",
        "Mai",
        "Jun",
        "Jul",
        "Aug",
        "Sep",
        "Okt",
        "Nov",
        "Dez",
    ]

    month_names = month_names_de if locale == "de" else month_names_en

    def is_first_of_month(d: date) -> bool:
        return d.day == 1

    def is_last_of_month(d: date) -> bool:
        next_day = d + timedelta(days=1)
        return next_day.month != d.month

    def is_first_of_year(d: date) -> bool:
        return d.month == 1 and d.day == 1

    def is_last_of_year(d: date) -> bool:
        return d.month == 12 and d.day == 31

    # Check for whole month
    if (
        is_first_of_month(start)
        and is_last_of_month(end)
        and start.month == end.month
        and start.year == end.year
    ):
        return f"{month_names[start.month - 1]} {start.year}"

    # Check for whole quarter
    if is_first_of_year(start) and is_last_of_year(end) and start.year == end.year:
        return str(start.year)

    quarters = [(1, 3), (4, 6), (7, 9), (10, 12)]
    for q_idx, (q_start_month, q_end_month) in enumerate(quarters, 1):
        q_start = date(start.year, q_start_month, 1)
        if q_end_month == 12:
            q_end = date(start.year, 12, 31)
        else:
            q_end = date(start.year, q_end_month + 1, 1) - timedelta(days=1)

        if start == q_start and end == q_end:
            return f"Q{q_idx} {start.year}"

    # Check for whole year
    if is_first_of_year(start) and is_last_of_year(end) and start.year == end.year:
        return str(start.year)

    # Same day
    if start == end:
        return f"{start.day} {month_names[start.month - 1]} {start.year}"

    # Same month
    if start.month == end.month and start.year == end.year:
        return f"{start.day}-{end.day} {month_names[start.month - 1]} {start.year}"

    # Same year
    if start.year == end.year:
        return f"{start.day} {month_names[start.month - 1]} - {end.day} {month_names[end.month - 1]} {start.year}"

    # Different years
    return f"{start.day} {month_names[start.month - 1]} {start.year} - {end.day} {month_names[end.month - 1]} {end.year}"
