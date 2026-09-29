"""IBAN-style mod-97 validation and formatting."""

import re

_PATTERN = re.compile(r"[A-Z]{2}[0-9]{2}[A-Z0-9]{11,30}")


def _normalize(value: str) -> str:
    return value.replace(" ", "").replace("-", "").upper()


def validate_iban(value: str) -> bool:
    """Check the mod-97 checksum of an IBAN-like string."""
    if not isinstance(value, str):
        return False
    candidate = _normalize(value)
    if not _PATTERN.fullmatch(candidate):
        return False
    rearranged = candidate[4:] + candidate[:4]
    digits = "".join(str(int(ch, 36)) for ch in rearranged)
    return int(digits) % 97 == 1


def format_iban(value: str) -> str:
    """Return the IBAN grouped in blocks of four characters."""
    if not validate_iban(value):
        raise ValueError(f"invalid IBAN: {value!r}")
    normalized = _normalize(value)
    return " ".join(normalized[i : i + 4] for i in range(0, len(normalized), 4))
