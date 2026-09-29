import re


def validate_and_normalize_email(email: str) -> str | None:
    """Validate and normalize email address."""
    email = email.strip().lower()

    # Basic email regex pattern
    pattern = r"^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$"

    if not re.match(pattern, email):
        return None

    # Basic normalization
    return email
