"""Normalise company names for duplicate matching."""

import re
import unicodedata

_LEGAL_SUFFIXES = {
    "ag", "gmbh", "sa", "sarl", "sagl", "kg", "ohg", "ug", "mbh", "ltd", "limited", "llc", "lp", "llp",
    "inc", "incorporated", "corp", "corporation", "co", "company", "plc", "bv", "nv", "srl", "spa",
    "oy", "ab", "as", "aps", "se", "cie",
}  # fmt: skip
_STOP_WORDS = {"the", "und", "and", "&"}


def normalize_company_name(name: str, *, drop_stop_words: bool = True) -> str:
    """Return a canonical key: lowercase, no accents/punctuation/legal suffixes."""
    text = unicodedata.normalize("NFKD", name.replace("&", " and ").replace("ß", "ss"))
    text = "".join(ch for ch in text if not unicodedata.combining(ch)).lower()
    text = re.sub(r"[^a-z0-9]+", " ", text)
    tokens = text.split()
    # Strip legal-form suffixes (possibly several, e.g. "Foo Holding AG & Co KG").
    while len(tokens) > 1 and tokens[-1] in _LEGAL_SUFFIXES:
        tokens.pop()
    if drop_stop_words:
        tokens = [t for t in tokens if t not in _STOP_WORDS] or tokens
    return " ".join(tokens)
