from typing import Any


def mask_sensitive(data: Any, sensitive_keys: set[str] | None = None) -> Any:
    """Mask sensitive values in nested data."""
    if sensitive_keys is None:
        sensitive_keys = {"password", "token", "secret", "card"}

    if isinstance(data, dict):
        result = {}
        for key, value in data.items():
            if key.lower() in sensitive_keys:
                result[key] = "***"
            else:
                result[key] = mask_sensitive(value, sensitive_keys)
        return result

    if isinstance(data, list):
        return [mask_sensitive(item, sensitive_keys) for item in data]

    return data
