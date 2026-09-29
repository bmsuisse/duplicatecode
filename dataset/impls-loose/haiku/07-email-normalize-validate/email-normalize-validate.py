import re


def normalize_email(email: str) -> str | None:
    """Validate and normalize an email address.

    Returns normalized email or None if invalid.
    """
    email = email.strip().lower()

    # Simple regex for email validation
    pattern = r"^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$"
    if not re.match(pattern, email):
        return None

    return email
