import re
from collections.abc import Mapping
from typing import Any


def validate_row(
    row: Mapping[str, Any], rules: Mapping[str, Mapping[str, Any]]
) -> dict[str, list[str]]:
    errors: dict[str, list[str]] = {}

    for field, rule in rules.items():
        field_errors: list[str] = []
        value = row.get(field)

        # Check if empty
        is_empty = value is None or value == ""

        # Check required
        if rule.get("required", False) and is_empty:
            field_errors.append("required")
            errors[field] = field_errors
            continue

        # If empty and not required, skip other checks (except one_of which still applies)
        if is_empty:
            if "one_of" in rule and value not in rule["one_of"]:
                field_errors.append("one_of")
            if field_errors:
                errors[field] = field_errors
            continue

        # Check type
        type_str = rule.get("type")
        if type_str:
            valid_type = False
            if type_str == "str":
                valid_type = isinstance(value, str)
            elif type_str == "int":
                valid_type = isinstance(value, int) and not isinstance(value, bool)
            elif type_str == "float":
                valid_type = isinstance(value, (int, float)) and not isinstance(value, bool)
            elif type_str == "bool":
                valid_type = isinstance(value, bool)

            if not valid_type:
                field_errors.append("type")
                errors[field] = field_errors
                continue

        # If type check passed, check min/max/pattern
        if type_str == "str":
            min_val = rule.get("min")
            if min_val is not None and len(value) < min_val:
                field_errors.append("min")

            max_val = rule.get("max")
            if max_val is not None and len(value) > max_val:
                field_errors.append("max")

            pattern = rule.get("pattern")
            if pattern is not None and not re.fullmatch(pattern, value):
                field_errors.append("pattern")

        elif type_str in ("int", "float"):
            min_val = rule.get("min")
            if min_val is not None and value < min_val:
                field_errors.append("min")

            max_val = rule.get("max")
            if max_val is not None and value > max_val:
                field_errors.append("max")

        # Check one_of
        if "one_of" in rule and value not in rule["one_of"]:
            field_errors.append("one_of")

        if field_errors:
            errors[field] = field_errors

    return errors
