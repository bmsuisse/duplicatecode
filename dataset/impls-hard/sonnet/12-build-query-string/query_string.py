"""Build URL query strings from dictionaries."""

from collections.abc import Iterable, Mapping
from typing import Any
from urllib.parse import quote


def _format_value(value: Any) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    return str(value)


def build_query_string(params: Mapping[str, Any], *, prefix: bool = False) -> str:
    """Return a percent-encoded query string such as ``a=1&b=x%20y``.

    ``None`` values are skipped, list/tuple/set values produce repeated keys and
    booleans become ``true``/``false``. With ``prefix`` a leading ``?`` is added
    (only when there is at least one parameter).
    """
    parts: list[str] = []
    for key, value in params.items():
        if value is None:
            continue
        values: Iterable[Any] = value if isinstance(value, list | tuple | set) else [value]
        for item in values:
            if item is None:
                continue
            parts.append(f"{quote(str(key), safe='')}={quote(_format_value(item), safe='')}")
    query = "&".join(parts)
    return f"?{query}" if prefix and query else query
