import re
import unicodedata


def normalize_org_name(name: str) -> str:
    if not name:
        return ""

    # Casefold
    text = name.casefold()

    # Replace & with ' and '
    text = text.replace("&", " and ")

    # Transliterate accents using NFKD
    text = unicodedata.normalize("NFKD", text)
    text = "".join(c for c in text if not unicodedata.combining(c))

    # Drop punctuation (keep only alphanumeric and space)
    text = re.sub(r"[^a-z0-9\s]", "", text)

    # Collapse whitespace
    text = re.sub(r"\s+", " ", text).strip()

    # Remove trailing legal-form tokens
    legal_forms = {
        "ag",
        "gmbh",
        "sa",
        "sarl",
        "ltd",
        "limited",
        "inc",
        "llc",
        "co",
        "corp",
        "plc",
        "bv",
        "srl",
        "spa",
    }

    tokens = text.split()
    while tokens and tokens[-1] in legal_forms and len(tokens) > 1:
        tokens.pop()

    return " ".join(tokens)


def org_names_match(a: str, b: str) -> bool:
    norm_a = normalize_org_name(a)
    norm_b = normalize_org_name(b)
    return norm_a == norm_b and norm_a != ""
