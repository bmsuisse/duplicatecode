import re


def normalize_phone(raw: str, default_country_code: str = "41") -> str | None:
    # Strip allowed separators
    stripped = raw
    for char in [" ", ".", "-", "(", ")", "/"]:
        stripped = stripped.replace(char, "")

    if not stripped:
        return None

    # Check for invalid characters (letters or other chars)
    if not re.match(r"^[+0-9]+$", stripped):
        return None

    # Handle prefixes
    if stripped.startswith("+"):
        digits = stripped[1:]
    elif stripped.startswith("00"):
        digits = stripped[2:]
    elif stripped.startswith("0"):
        digits = default_country_code + stripped[1:]
    else:
        return None

    # Drop '(0)' trunk marker after country code
    # This appears as (0) directly following digits
    digits = re.sub(r"(\d)\(0\)", r"", digits)

    # Check digit count
    if len(digits) < 8 or len(digits) > 15:
        return None

    return "+" + digits
