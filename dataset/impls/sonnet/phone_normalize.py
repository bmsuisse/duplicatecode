"""Phone number normalisation to an E.164-like form."""

import re

_SEPARATORS = re.compile(r"[\s.\-()/]")
_TRUNK = re.compile(r"\(\s*0\s*\)")


def normalize_phone(raw: str, default_country_code: str = "41") -> str | None:
    """Normalise ``raw`` to ``+<digits>`` or return None if it is not recognisable."""
    text = _TRUNK.sub("", raw.strip())
    text = _SEPARATORS.sub("", text)
    if not text:
        return None
    if text.startswith("+"):
        digits = text[1:]
    elif text.startswith("00"):
        digits = text[2:]
    elif text.startswith("0"):
        digits = default_country_code + text[1:]
    else:
        return None
    if not digits.isascii() or not digits.isdigit():
        return None
    if not 8 <= len(digits) <= 15:
        return None
    return "+" + digits
