import re


def normalize_company_name(name: str) -> str:
    """Normalize company name for deduplication."""
    # Convert to lowercase
    name = name.lower().strip()

    # Remove legal form suffixes
    suffixes = [
        r"\b(ag|ltd|llc|inc|corp|gmbh|sarl|pvt|co\.|s\.a\.|s\.l\.)\b",
        r"\binc\.?\b",
        r"\bco\.,? ?inc\.?\b",
    ]

    for suffix in suffixes:
        name = re.sub(suffix, "", name)

    # Remove punctuation and extra spaces
    name = re.sub(r"[^a-z0-9\s]", "", name)
    name = re.sub(r"\s+", " ", name)

    return name.strip()
