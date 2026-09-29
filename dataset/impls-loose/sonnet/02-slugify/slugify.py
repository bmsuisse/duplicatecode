"""Turn arbitrary text into URL slugs."""

import re
import unicodedata

_SPECIAL = str.maketrans(
    {"ß": "ss", "æ": "ae", "Æ": "AE", "ø": "o", "Ø": "O", "œ": "oe", "Œ": "OE", "đ": "d", "ł": "l"}
)
_NON_ALNUM = re.compile(r"[^a-z0-9]+")


def slugify(text: str, separator: str = "-", max_length: int | None = None) -> str:
    """Return a lowercase, ASCII, ``separator``-delimited slug for ``text``."""
    text = text.translate(_SPECIAL)
    text = unicodedata.normalize("NFKD", text)
    text = text.encode("ascii", "ignore").decode("ascii").lower()
    slug = _NON_ALNUM.sub(separator, text).strip(separator)
    if max_length is not None and len(slug) > max_length:
        slug = slug[:max_length].rstrip(separator)
    return slug
