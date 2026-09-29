"""Convert free text into a URL-safe slug."""

import re
import unicodedata

_SPECIAL_LETTERS = str.maketrans(
    {"ß": "ss", "æ": "ae", "Æ": "AE", "ø": "o", "Ø": "O", "œ": "oe", "Œ": "OE", "đ": "d", "ł": "l"}
)
_NON_ALNUM = re.compile(r"[^a-z0-9]+")


def slugify(text: str, *, separator: str = "-", max_length: int | None = None) -> str:
    """Return a lowercase ASCII slug for ``text``.

    Accents are stripped, umlauts are transliterated to their base letter and
    every run of other characters becomes a single ``separator``.
    """
    if max_length is not None and max_length < 1:
        raise ValueError("max_length must be positive")
    text = text.translate(_SPECIAL_LETTERS)
    text = unicodedata.normalize("NFKD", text)
    text = text.encode("ascii", "ignore").decode("ascii").lower()
    slug = _NON_ALNUM.sub(separator, text).strip(separator)
    if max_length is not None:
        slug = slug[:max_length].strip(separator)
    return slug
