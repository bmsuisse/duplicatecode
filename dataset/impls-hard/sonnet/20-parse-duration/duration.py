"""Parse human-written durations such as ``1h 30m`` into seconds."""

import re

_UNITS = {
    "w": 604800,
    "d": 86400,
    "h": 3600,
    "m": 60,
    "s": 1,
}
_ALIASES = {
    "week": "w", "weeks": "w", "wk": "w",
    "day": "d", "days": "d",
    "hour": "h", "hours": "h", "hr": "h", "hrs": "h",
    "minute": "m", "minutes": "m", "min": "m", "mins": "m",
    "second": "s", "seconds": "s", "sec": "s", "secs": "s",
}  # fmt: skip
_PART = re.compile(r"(\d+(?:\.\d+)?)\s*([a-z]+)")


def parse_duration(text: str) -> float:
    """Return the total number of seconds described by ``text``.

    Examples: ``"90"`` (plain seconds), ``"1h30m"``, ``"2 days 4 hours"``,
    ``"1.5h"``. Raises ``ValueError`` for anything not understood.
    """
    source = text.strip().lower().replace(",", " ")
    if not source:
        raise ValueError("empty duration")
    if re.fullmatch(r"\d+(?:\.\d+)?", source):
        return float(source)
    total = 0.0
    position = 0
    while position < len(source):
        if source[position].isspace():
            position += 1
            continue
        match = _PART.match(source, position)
        if match is None:
            raise ValueError(f"invalid duration: {text!r}")
        unit = _ALIASES.get(match.group(2), match.group(2))
        if unit not in _UNITS:
            raise ValueError(f"unknown unit {match.group(2)!r} in duration: {text!r}")
        total += float(match.group(1)) * _UNITS[unit]
        position = match.end()
    return total
