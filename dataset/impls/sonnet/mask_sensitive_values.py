"""Mask sensitive values in nested data."""

from collections.abc import Collection
from typing import Any


def mask_sensitive(
    data: Any,
    *,
    keys: Collection[str] = ("password", "token", "secret", "authorization"),
    mask: str = "***",
) -> Any:
    """Return a deep copy of ``data`` with sensitive dict values replaced by ``mask``."""
    needles = [k.lower() for k in keys]

    def is_sensitive(key: Any) -> bool:
        return isinstance(key, str) and any(n in key.lower() for n in needles)

    def walk(node: Any) -> Any:
        if isinstance(node, dict):
            return {k: mask if is_sensitive(k) else walk(v) for k, v in node.items()}
        if isinstance(node, list):
            return [walk(v) for v in node]
        if isinstance(node, tuple):
            return tuple(walk(v) for v in node)
        return node

    return walk(data)
