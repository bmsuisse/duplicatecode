"""Mask sensitive values in nested data before logging."""

import re
from collections.abc import Collection, Mapping
from typing import Any

DEFAULT_KEYS = (
    "password",
    "passwd",
    "secret",
    "token",
    "api_key",
    "apikey",
    "authorization",
    "credential",
    "private_key",
    "ssn",
    "card",
)


def mask_sensitive(
    data: Any,
    *,
    sensitive_keys: Collection[str] = DEFAULT_KEYS,
    mask: str = "***",
    _depth: int = 0,
) -> Any:
    """Return a copy of ``data`` with values under sensitive keys replaced by ``mask``.

    A key is sensitive if it contains any of ``sensitive_keys`` (case-insensitive,
    ``-`` and ``_`` treated alike). Dicts, lists, tuples and sets are walked.
    """
    patterns = [re.sub(r"[-_]", "", k.lower()) for k in sensitive_keys]

    def is_sensitive(key: Any) -> bool:
        normalized = re.sub(r"[-_]", "", str(key).lower())
        return any(p in normalized for p in patterns)

    def walk(value: Any, depth: int) -> Any:
        if depth > 50:
            return mask
        if isinstance(value, Mapping):
            return {k: mask if is_sensitive(k) else walk(v, depth + 1) for k, v in value.items()}
        if isinstance(value, list):
            return [walk(v, depth + 1) for v in value]
        if isinstance(value, tuple):
            return tuple(walk(v, depth + 1) for v in value)
        if isinstance(value, set | frozenset):
            return type(value)(walk(v, depth + 1) for v in value)
        return value

    return walk(data, _depth)
