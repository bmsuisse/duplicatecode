"""Convert text into a URL-friendly slug."""

import re
import unicodedata

_EXTRA_MAP = {"ß": "ss", "æ": "ae", "ø": "o"}
_NON_ALNUM = re.compile(r"[^a-z0-9]+")


def slugify(text: str, *, max_length: int | None = None, separator: str = "-") -> str:
    """Return a lowercase ASCII slug of ``text``."""
    lowered = text.lower()
    for source, target in _EXTRA_MAP.items():
        lowered = lowered.replace(source, target)
    decomposed = unicodedata.normalize("NFKD", lowered)
    stripped = "".join(ch for ch in decomposed if not unicodedata.combining(ch))
    slug = _NON_ALNUM.sub(separator, stripped).strip(separator)
    if max_length is not None:
        slug = slug[: max(max_length, 0)].rstrip(separator)
    return slug
