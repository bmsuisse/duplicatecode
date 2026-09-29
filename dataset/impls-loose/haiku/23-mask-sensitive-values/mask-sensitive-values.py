from typing import Any


def mask_sensitive(
    data: Any,
    sensitive_keys: set[str],
    mask: str = "***",
) -> Any:
    """Mask sensitive values in nested data structures."""
    if isinstance(data, dict):
        result = {}
        for key, value in data.items():
            if key.lower() in {k.lower() for k in sensitive_keys}:
                result[key] = mask
            else:
                result[key] = mask_sensitive(value, sensitive_keys, mask)
        return result
    elif isinstance(data, list):
        return [mask_sensitive(item, sensitive_keys, mask) for item in data]
    else:
        return data
