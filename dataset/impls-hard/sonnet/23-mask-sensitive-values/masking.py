"""Mask sensitive values in nested data before logging."""

import re
from collections.abc import Collection, Mapping
from typing import Any

DEFAULT_SENSITIVE_KEYS = frozenset(
    {"password", "passwd", "secret", "token", "api_key", "apikey", "authorization",
     "card_number", "cardnumber", "cvv", "pin", "access_token", "refresh_token"}
)  # fmt: skip
_CARD_NUMBER = re.compile(r"\b(?:\d[ -]?){13,19}\b")
_BEARER = re.compile(r"(?i)\b(bearer|basic)\s+[A-Za-z0-9._~+/=-]+")


def _mask_text(text: str, mask: str) -> str:
    text = _BEARER.sub(lambda m: f"{m.group(1)} {mask}", text)

    def replace_card(match: re.Match[str]) -> str:
        digits = re.sub(r"\D", "", match.group(0))
        return f"{mask}{digits[-4:]}" if 13 <= len(digits) <= 19 else match.group(0)

    return _CARD_NUMBER.sub(replace_card, text)


def mask_sensitive(
    data: Any,
    sensitive_keys: Collection[str] = DEFAULT_SENSITIVE_KEYS,
    mask: str = "***",
) -> Any:
    """Return a copy of ``data`` with sensitive values replaced by ``mask``.

    Dict values whose key (case-insensitive) is in ``sensitive_keys`` or contains
    one of them are masked entirely. Strings elsewhere have card numbers reduced
    to their last four digits and bearer tokens hidden. Dicts, lists and tuples
    are traversed recursively; the original is never modified.
    """
    keys = {k.lower() for k in sensitive_keys}
    return _walk(data, keys, mask)


def _is_sensitive(key: object, keys: set[str]) -> bool:
    lowered = str(key).lower()
    return any(k in lowered for k in keys)


def _walk(value: Any, keys: set[str], mask: str) -> Any:
    if isinstance(value, Mapping):
        return {
            k: mask if _is_sensitive(k, keys) and v is not None else _walk(v, keys, mask)
            for k, v in value.items()
        }
    if isinstance(value, list):
        return [_walk(v, keys, mask) for v in value]
    if isinstance(value, tuple):
        return tuple(_walk(v, keys, mask) for v in value)
    if isinstance(value, str):
        return _mask_text(value, mask)
    return value
