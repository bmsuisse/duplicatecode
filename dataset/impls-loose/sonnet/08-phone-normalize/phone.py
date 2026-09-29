"""Normalise user-entered phone numbers to an E.164-like form."""

import re

_MIN_DIGITS = 7
_MAX_DIGITS = 15


def normalize_phone(
    raw: str, default_country_code: str = "41", trunk_prefix: str = "0"
) -> str | None:
    """Return ``+<country><number>`` or None if the input can't be a phone number.

    Handles spaces, dashes, brackets, ``00`` international prefixes and national
    numbers with a leading trunk ``0`` (resolved against ``default_country_code``).
    """
    text = raw.strip()
    if not text:
        return None
    # Drop extensions such as "ext. 12" or "x12".
    text = re.split(r"\s*(?:ext\.?|x)\s*\d+$", text, maxsplit=1, flags=re.IGNORECASE)[0]
    if re.search(r"[^\d\s()+./-]", text):
        return None
    international = text.startswith("+")
    digits = re.sub(r"\D", "", text)
    if not international and digits.startswith("00"):
        international = True
        digits = digits[2:]
    if not international:
        # "+41 (0) 79 ..." style is handled below; here it's a national number.
        digits = default_country_code + digits.removeprefix(trunk_prefix)
    else:
        # Remove the redundant "(0)" trunk digit after a country code.
        match = re.match(r"^\+?\s*(\d{1,3})\s*\(0\)", text)
        if match:
            digits = match.group(1) + digits[len(match.group(1)) + 1 :]
    if not _MIN_DIGITS <= len(digits) <= _MAX_DIGITS:
        return None
    return "+" + digits
