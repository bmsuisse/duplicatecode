"""Parse human duration strings into seconds."""

import re

_UNITS = {"ms": 0.001, "s": 1, "m": 60, "h": 3600, "d": 86400, "w": 604800}
_PART = re.compile(r"(\d+(?:\.\d+)?)\s*([a-z]+)")


def parse_duration(text: str) -> float:
    """Return the total number of seconds described by ``text``."""
    source = text.strip().lower()
    if not source:
        raise ValueError("empty duration")
    total = 0.0
    seen: set[str] = set()
    position = 0
    while position < len(source):
        match = _PART.match(source, position)
        if match is None:
            raise ValueError(f"invalid duration: {text!r}")
        amount, unit = match.groups()
        if unit not in _UNITS:
            raise ValueError(f"unknown unit {unit!r} in {text!r}")
        if unit in seen:
            raise ValueError(f"repeated unit {unit!r} in {text!r}")
        seen.add(unit)
        total += float(amount) * _UNITS[unit]
        position = match.end()
        while position < len(source) and source[position].isspace():
            position += 1
    return total
