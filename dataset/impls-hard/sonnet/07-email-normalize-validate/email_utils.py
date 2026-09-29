"""Plausibility check and normalisation for e-mail addresses."""

import re

_LOCAL_PART = re.compile(r"[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*")
_DOMAIN_LABEL = re.compile(r"[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?")


def normalize_email(value: str) -> str:
    """Trim surrounding whitespace and lower-case the address."""
    return value.strip().lower()


def is_valid_email(value: str) -> bool:
    """Return True if the address looks plausible after normalisation."""
    email = normalize_email(value)
    if len(email) > 254 or email.count("@") != 1:
        return False
    local, domain = email.split("@")
    if not local or len(local) > 64 or not _LOCAL_PART.fullmatch(local):
        return False
    labels = domain.split(".")
    if len(labels) < 2 or not all(_DOMAIN_LABEL.fullmatch(label) for label in labels):
        return False
    return not labels[-1].isdigit()


def clean_email(value: str) -> str:
    """Return the normalised address or raise ``ValueError`` if it is invalid."""
    email = normalize_email(value)
    if not is_valid_email(email):
        raise ValueError(f"invalid email address: {value!r}")
    return email
