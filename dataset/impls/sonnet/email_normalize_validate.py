"""Email validation and normalisation."""

import re

_LOCAL = re.compile(r"[A-Za-z0-9.!#$%&'*+/=?^_`{|}~-]{1,64}")
_LABEL = re.compile(r"[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?")
_TLD = re.compile(r"[A-Za-z]{2,}")


def normalize_email(value: str) -> str | None:
    """Return the email with a lowercased domain, or None when invalid."""
    email = value.strip()
    if len(email) > 254 or email.count("@") != 1:
        return None
    local, domain = email.split("@")
    if not _LOCAL.fullmatch(local):
        return None
    if local.startswith(".") or local.endswith(".") or ".." in local:
        return None
    labels = domain.split(".")
    if len(labels) < 2 or not all(_LABEL.fullmatch(label) for label in labels):
        return None
    if not _TLD.fullmatch(labels[-1]):
        return None
    return f"{local}@{domain.lower()}"
