from datetime import date


def format_period_label(start: date, end: date, locale: str = "en") -> str:
    """Format a date period as a label."""
    months_en = [
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
    ]
    months_de = [
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
    ]

    months = months_de if locale == "de" else months_en

    if start.year == end.year and start.month == end.month:
        return f"{months[start.month - 1]} {start.year}"
    elif start.year == end.year:
        return f"{months[start.month - 1]} - {months[end.month - 1]} {start.year}"
    else:
        return f"{months[start.month - 1]} {start.year} - {months[end.month - 1]} {end.year}"
