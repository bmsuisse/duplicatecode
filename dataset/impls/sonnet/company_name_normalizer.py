"""Normalise organisation names for duplicate matching."""

import re
import unicodedata

_LEGAL_FORMS = {
    "ag", "gmbh", "sa", "sarl", "ltd", "limited", "inc",
    "llc", "co", "corp", "plc", "bv", "srl", "spa",
}  # fmt: skip


def normalize_org_name(name: str) -> str:
    """Return a canonical lowercase form of an organisation name."""
    text = unicodedata.normalize("NFKD", name.casefold())
    text = "".join(ch for ch in text if not unicodedata.combining(ch))
    text = text.replace("&", " and ")
    text = re.sub(r"[^\w\s]|_", "", text)
    tokens = text.split()
    while len(tokens) > 1 and tokens[-1] in _LEGAL_FORMS:
        tokens.pop()
    return " ".join(tokens)


def org_names_match(a: str, b: str) -> bool:
    """True when both names normalise to the same non-empty string."""
    normalized = normalize_org_name(a)
    return bool(normalized) and normalized == normalize_org_name(b)
