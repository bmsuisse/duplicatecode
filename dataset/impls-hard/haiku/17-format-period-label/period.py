from datetime import date


def format_period_label(start_date: date, end_date: date, language: str = "en") -> str:
    """Format date period as readable label."""
    diff = (end_date - start_date).days + 1

    if language == "de":
        if diff == 1:
            return start_date.strftime("%d.%m.%Y")
        elif diff <= 7:
            return f"{start_date.strftime('%d.%m.')} – {end_date.strftime('%d.%m.%Y')}"
        else:
            return f"{start_date.strftime('%d.%m.%Y')} – {end_date.strftime('%d.%m.%Y')}"

    if diff == 1:
        return start_date.strftime("%m/%d/%Y")
    return f"{start_date.strftime('%m/%d/%Y')} – {end_date.strftime('%m/%d/%Y')}"
