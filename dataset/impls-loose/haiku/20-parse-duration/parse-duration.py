import re


def parse_duration(duration_str: str) -> int:
    """Parse human-friendly duration string to seconds."""
    duration_str = duration_str.lower().strip()
    seconds = 0

    # Match patterns like "1d", "2h", "30m", "45s"
    pattern = r"(\d+)\s*(d|day|days|h|hour|hours|m|min|minute|minutes|s|sec|second|seconds)"

    for match in re.finditer(pattern, duration_str):
        value = int(match.group(1))
        unit = match.group(2)[0].lower()  # Get first character

        if unit == "d":
            seconds += value * 86400
        elif unit == "h":
            seconds += value * 3600
        elif unit == "m":
            seconds += value * 60
        elif unit == "s":
            seconds += value

    return seconds
