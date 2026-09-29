import re
import unicodedata


def normalize_company_name(name: str) -> str:
    """Normalize company name for duplicate matching."""
    # Convert to lowercase
    name = name.lower()

    # Normalize unicode
    name = unicodedata.normalize("NFKD", name)
    name = name.encode("ascii", "ignore").decode("ascii")

    # Remove common legal suffixes
    suffixes = [
        r"\b(ltd|limited|llc|inc|incorporated|corp|corporation|ag|sa|gmbh|bv|nv|plc)\b\.?",
        r"\b(and|&)\b",
    ]
    for suffix in suffixes:
        name = re.sub(suffix, "", name)

    # Remove punctuation except spaces
    name = re.sub(r"[^a-z0-9\s]", "", name)

    # Normalize whitespace
    name = re.sub(r"\s+", " ", name).strip()

    return name
