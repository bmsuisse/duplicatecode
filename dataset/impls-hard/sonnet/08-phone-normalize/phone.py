"""Normalise phone numbers to the international E.164 notation."""

import re

_ALLOWED = re.compile(r"[\d\s()./-]*")


def normalize_phone(value: str, default_country_code: str = "41") -> str:
    """Return ``value`` as ``+<country code><number>``.

    Accepts separators (spaces, dots, dashes, slashes, parentheses), a leading
    ``+`` or ``00`` for international numbers and a leading ``0`` for national
    numbers, which is replaced by ``default_country_code``. Raises
    ``ValueError`` for input that cannot be a phone number.
    """
    text = value.strip()
    international = text.startswith("+")
    body = text[1:] if international else text
    if not _ALLOWED.fullmatch(body):
        raise ValueError(f"invalid phone number: {value!r}")
    digits = re.sub(r"\D", "", body)
    if not international and digits.startswith("00"):
        international = True
        digits = digits[2:]
    if international:
        # Drop a redundant trunk zero such as +41 (0)79 ...
        code = default_country_code
        if digits.startswith(code + "0"):
            digits = code + digits[len(code) + 1 :]
    elif digits.startswith("0"):
        digits = default_country_code + digits[1:]
    else:
        raise ValueError(f"cannot determine country code: {value!r}")
    if not 8 <= len(digits) <= 15 or digits.startswith("0"):
        raise ValueError(f"invalid phone number: {value!r}")
    return "+" + digits
