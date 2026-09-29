from typing import Any
from urllib.parse import urlencode


def build_query_string(params: dict[str, Any]) -> str:
    """Build a URL query string with proper escaping."""
    return urlencode(params)
