from collections.abc import Callable
from typing import Any


def validate_row(
    row: dict[str, Any],
    rules: dict[str, Callable[[Any], bool]],
) -> dict[str, list[str]]:
    """Validate a row against declarative rules."""
    errors: dict[str, list[str]] = {}

    for field, validator in rules.items():
        if field not in row:
            if "required" in str(validator):
                errors[field] = ["Field is required"]
        else:
            try:
                if not validator(row[field]):
                    errors[field] = ["Validation failed"]
            except (ValueError, TypeError, AttributeError, KeyError) as e:
                errors[field] = [str(e)]

    return errors
