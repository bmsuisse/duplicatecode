"""Email validation and normalisation."""

import re

_LOCAL = re.compile(r"^[A-Za-z0-9!#$%&'*+/=?^_`{|}~-]+(\.[A-Za-z0-9!#$%&'*+/=?^_`{|}~-]+)*$")
_LABEL = re.compile(r"^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$")


def normalize_email(email: str) -> str | None:
    """Return the trimmed, case-normalised address, or None if it looks invalid."""
    candidate = email.strip()
    if candidate.count("@") != 1:
        return None
    local, domain = candidate.split("@")
    domain = domain.lower().rstrip(".")
    if not local or len(local) > 64 or len(local) + len(domain) + 1 > 254:
        return None
    if not _LOCAL.match(local):
        return None
    try:
        domain = domain.encode("idna").decode("ascii")
    except UnicodeError:
        return None
    labels = domain.split(".")
    if len(labels) < 2 or not all(_LABEL.match(label) for label in labels):
        return None
    if labels[-1].isdigit():
        return None
    return f"{local}@{domain}"


def is_valid_email(email: str) -> bool:
    return normalize_email(email) is not None
