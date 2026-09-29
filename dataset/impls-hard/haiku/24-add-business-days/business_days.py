from datetime import date, timedelta


def add_business_days(start_date: date, days: int) -> date:
    """Add business days to date excluding weekends."""
    current = start_date
    days_added = 0
    direction = 1 if days >= 0 else -1
    days_to_add = abs(days)

    while days_added < days_to_add:
        current += timedelta(days=direction)
        # Monday=0, Sunday=6
        if current.weekday() < 5:  # 0-4 are weekdays
            days_added += 1

    return current
