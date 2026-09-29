import re


def normalize_phone(phone: str) -> str:
    """Normalize phone number to E.164-like format."""
    # Remove all non-digit characters
    digits = re.sub(r"\D", "", phone)

    # If it starts with 1, assume US, keep as is
    # Otherwise assume international format
    if not digits:
        return ""

    # Remove leading 0 if present (common in many countries)
    if len(digits) > 1 and digits[0] == "0":
        digits = digits[1:]

    # If no country code, assume +41 (Switzerland)
    if len(digits) == 9 or (len(digits) == 10 and not phone.strip().startswith("+")):
        return f"+41{digits[-9:]}"

    # Add + if not present
    if not digits.startswith("+"):
        return f"+{digits}"

    return f"+{digits}"
