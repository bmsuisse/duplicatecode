import re


def parse_duration(duration_str: str) -> int:
    """Parse duration string to seconds."""
    duration_str = duration_str.strip().lower()

    # Pattern: number + unit
    pattern = r"(\d+)\s*(s|second|m|minute|h|hour|d|day|w|week)"

    total_seconds = 0
    units = {
        "s": 1,
        "second": 1,
        "m": 60,
        "minute": 60,
        "h": 3600,
        "hour": 3600,
        "d": 86400,
        "day": 86400,
        "w": 604800,
        "week": 604800,
    }

    matches = re.findall(pattern, duration_str)
    if not matches:
        raise ValueError(f"Invalid duration: {duration_str}")

    for value, unit in matches:
        total_seconds += int(value) * units.get(unit, 0)

    return total_seconds
