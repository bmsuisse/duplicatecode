"""Build URL query strings."""

from collections.abc import Mapping
from typing import Any
from urllib.parse import quote


def _to_text(value: Any) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    return str(value)


def build_query_string(params: Mapping[str, Any], *, sort_keys: bool = False) -> str:
    """Encode ``params`` as ``a=1&b=2``. ``None`` values are skipped.

    List/tuple/set values are repeated as ``k=v1&k=v2``. No leading ``?``.
    """
    keys = sorted(params) if sort_keys else list(params)
    parts: list[str] = []
    for key in keys:
        value = params[key]
        if value is None:
            continue
        values = value if isinstance(value, list | tuple | set | frozenset) else [value]
        parts.extend(
            f"{quote(str(key), safe='')}={quote(_to_text(v), safe='')}"
            for v in values
            if v is not None
        )
    return "&".join(parts)
