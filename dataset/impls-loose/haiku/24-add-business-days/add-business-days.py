from datetime import date, timedelta


def add_business_days(start_date: date, days: int) -> date:
    """Add business days to a date, skipping weekends."""
    current = start_date
    direction = 1 if days >= 0 else -1
    remaining = abs(days)

    while remaining > 0:
        current += timedelta(days=direction)
        # 0 = Monday, 6 = Sunday
        if current.weekday() < 5:  # Monday to Friday
            remaining -= 1

    return current
