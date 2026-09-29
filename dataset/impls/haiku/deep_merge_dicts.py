from typing import Any, Literal


def deep_merge(
    base: dict[str, Any],
    override: dict[str, Any],
    *,
    list_strategy: Literal["replace", "concat", "unique"] = "replace",
) -> dict[str, Any]:
    result = {}

    # Add all keys from base
    for key, value in base.items():
        if isinstance(value, (dict, list)):
            result[key] = value.copy() if hasattr(value, "copy") else value[:]
        else:
            result[key] = value

    # Merge in override
    for key, override_value in override.items():
        if key not in result:
            # New key from override
            if isinstance(override_value, (dict, list)):
                result[key] = (
                    override_value.copy() if hasattr(override_value, "copy") else override_value[:]
                )
            else:
                result[key] = override_value
        else:
            base_value = result[key]
            # Key exists in both
            if isinstance(base_value, dict) and isinstance(override_value, dict):
                # Recursively merge
                result[key] = deep_merge(base_value, override_value, list_strategy=list_strategy)
            elif isinstance(base_value, list) and isinstance(override_value, list):
                # Apply list strategy
                if list_strategy == "replace":
                    result[key] = override_value.copy()
                elif list_strategy == "concat":
                    result[key] = base_value + override_value
                elif list_strategy == "unique":
                    seen = set()
                    unique_list = []
                    for item in base_value + override_value:
                        try:
                            if item not in seen:
                                seen.add(item)
                                unique_list.append(item)
                        except TypeError:
                            # Unhashable type, use == comparison
                            if item not in unique_list:
                                unique_list.append(item)
                    result[key] = unique_list
            else:
                # Type mismatch or scalar override wins
                if isinstance(override_value, (dict, list)):
                    result[key] = (
                        override_value.copy()
                        if hasattr(override_value, "copy")
                        else override_value[:]
                    )
                else:
                    result[key] = override_value

    return result
