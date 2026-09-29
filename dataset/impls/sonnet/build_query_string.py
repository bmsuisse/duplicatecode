"""Build a URL query string."""

from collections.abc import Mapping
from typing import Any, Literal
from urllib.parse import quote


def _scalar(value: Any) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    return quote(str(value), safe="")


def build_query_string(
    params: Mapping[str, Any],
    *,
    array_format: Literal["repeat", "comma", "brackets"] = "repeat",
) -> str:
    """Encode ``params`` as ``k=v&k=v`` without a leading question mark."""
    parts: list[str] = []
    for key, value in params.items():
        if value is None:
            continue
        name = quote(key, safe="")
        if isinstance(value, (list, tuple)):
            if not value:
                continue
            encoded = [_scalar(v) for v in value]
            if array_format == "comma":
                parts.append(f"{name}={','.join(encoded)}")
            else:
                suffix = "[]" if array_format == "brackets" else ""
                parts.extend(f"{name}{suffix}={e}" for e in encoded)
        else:
            parts.append(f"{name}={_scalar(value)}")
    return "&".join(parts)
