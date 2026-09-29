"""Normalise company names so different spellings compare equal."""

import re
import unicodedata

_LEGAL_FORMS = {
    "ag", "gmbh", "sa", "sarl", "sagl", "kg", "ohg", "ug", "mbh", "co", "kgaa",
    "inc", "incorporated", "corp", "corporation", "ltd", "limited", "llc", "llp",
    "plc", "bv", "nv", "srl", "spa", "ab", "as", "oy", "company", "cie",
}  # fmt: skip
_TRANSLIT = str.maketrans({"ß": "ss", "æ": "ae", "ø": "o", "œ": "oe"})


def normalize_company_name(name: str, *, strip_legal_form: bool = True) -> str:
    """Return a lower-case, accent-free, punctuation-free form of ``name``.

    ``&`` becomes ``and``, umlauts are transliterated (``ü`` -> ``u``) and
    trailing legal-form tokens (AG, GmbH, Ltd, Inc, ...) are removed.
    """
    text = name.lower().translate(_TRANSLIT).replace("&", " and ")
    text = unicodedata.normalize("NFKD", text)
    text = "".join(ch for ch in text if not unicodedata.combining(ch))
    text = re.sub(r"[^\w\s]", " ", text)
    tokens = text.split()
    if strip_legal_form:
        while len(tokens) > 1 and tokens[-1] in _LEGAL_FORMS:
            tokens.pop()
    return " ".join(tokens)
