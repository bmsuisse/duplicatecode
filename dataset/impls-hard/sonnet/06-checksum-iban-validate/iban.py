"""IBAN validation (ISO 13616 / mod-97) and display formatting."""

import re

_IBAN_SHAPE = re.compile(r"[A-Z]{2}[0-9]{2}[A-Z0-9]{11,30}")


def normalize_iban(value: str) -> str:
    """Strip whitespace and hyphens and upper-case the value."""
    return re.sub(r"[\s-]+", "", value).upper()


def is_valid_iban(value: str) -> bool:
    """Return True if ``value`` has a plausible shape and a correct check sum."""
    iban = normalize_iban(value)
    if len(iban) > 34 or not _IBAN_SHAPE.fullmatch(iban):
        return False
    rearranged = iban[4:] + iban[:4]
    digits = "".join(str(int(ch, 36)) for ch in rearranged)
    return int(digits) % 97 == 1


def format_iban(value: str) -> str:
    """Return the IBAN in groups of four characters separated by spaces.

    Raises ``ValueError`` if the input is not a valid IBAN.
    """
    iban = normalize_iban(value)
    if not is_valid_iban(iban):
        raise ValueError(f"invalid IBAN: {value!r}")
    return " ".join(iban[i : i + 4] for i in range(0, len(iban), 4))
