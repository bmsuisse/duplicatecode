import re


def normalize_phone(phone: str) -> str:
    """Normalize phone number to E.164-like format."""
    # Remove all non-digit characters except leading +
    if phone.startswith("+"):
        phone = "+" + re.sub(r"\D", "", phone)
    else:
        phone = re.sub(r"\D", "", phone)

    # Ensure it starts with +
    if not phone.startswith("+"):
        phone = "+" + phone

    return phone
