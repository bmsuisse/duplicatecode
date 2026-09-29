from collections.abc import Collection
from typing import Any


def mask_sensitive(
    data: Any,
    *,
    keys: Collection[str] = ("password", "token", "secret", "authorization"),
    mask: str = "***",
) -> Any:
    # Convert keys to lowercase for case-insensitive matching
    sensitive_keys_lower = {k.lower() for k in keys}

    def is_sensitive_key(key: Any) -> bool:
        if not isinstance(key, str):
            return False
        key_lower = key.lower()
        return any(sens_key in key_lower for sens_key in sensitive_keys_lower)

    def recurse(obj: Any) -> Any:
        if isinstance(obj, dict):
            result = {}
            for k, v in obj.items():
                if is_sensitive_key(k):
                    result[k] = mask
                else:
                    result[k] = recurse(v)
            return result
        elif isinstance(obj, list):
            return [recurse(item) for item in obj]
        elif isinstance(obj, tuple):
            return tuple(recurse(item) for item in obj)
        else:
            return obj

    return recurse(data)
