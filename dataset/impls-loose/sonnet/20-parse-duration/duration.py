"""Parse human-friendly duration strings into seconds."""

import re

_UNITS = {
    "ms": 0.001, "millisecond": 0.001, "milliseconds": 0.001,
    "s": 1, "sec": 1, "secs": 1, "second": 1, "seconds": 1,
    "m": 60, "min": 60, "mins": 60, "minute": 60, "minutes": 60,
    "h": 3600, "hr": 3600, "hrs": 3600, "hour": 3600, "hours": 3600,
    "d": 86400, "day": 86400, "days": 86400,
    "w": 604800, "week": 604800, "weeks": 604800,
}  # fmt: skip
_TOKEN = re.compile(r"\s*(\d+(?:\.\d+)?)\s*([a-z]*)\s*(?:,|and)?", re.IGNORECASE)
_CLOCK = re.compile(r"^(?:(\d+):)?(\d{1,2}):(\d{2})$")


def parse_duration(text: str) -> float:
    """Parse ``"1h 30m"``, ``"2 days, 3 hours"``, ``"1:30:00"`` or ``"90"`` (seconds).

    Raises ``ValueError`` for anything not fully understood.
    """
    text = text.strip()
    if not text:
        raise ValueError("empty duration")
    clock = _CLOCK.match(text)
    if clock:
        hours, minutes, seconds = (int(g or 0) for g in clock.groups())
        return float(hours * 3600 + minutes * 60 + seconds)
    total = 0.0
    pos = 0
    while pos < len(text):
        match = _TOKEN.match(text, pos)
        if not match or match.end() == pos:
            raise ValueError(f"invalid duration: {text!r}")
        number, unit = match.group(1), match.group(2).lower()
        if unit not in _UNITS and unit != "":
            raise ValueError(f"unknown unit {unit!r} in {text!r}")
        total += float(number) * _UNITS.get(unit, 1)
        pos = match.end()
    return total
