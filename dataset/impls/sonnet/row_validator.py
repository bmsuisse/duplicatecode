"""Validate a row of data against field rules."""

import re
from collections.abc import Mapping
from typing import Any

_TYPES: dict[str, Any] = {"str": str, "int": int, "float": (int, float), "bool": bool}


def _type_ok(value: Any, name: str) -> bool:
    if name in ("int", "float") and isinstance(value, bool):
        return False
    return isinstance(value, _TYPES[name])


def validate_row(
    row: Mapping[str, Any], rules: Mapping[str, Mapping[str, Any]]
) -> dict[str, list[str]]:
    """Return a mapping of field name to error codes for invalid fields."""
    result: dict[str, list[str]] = {}
    for field, rule in rules.items():
        value = row.get(field)
        if value is None or value == "":
            if rule.get("required"):
                result[field] = ["required"]
            continue
        errors: list[str] = []
        type_name = rule.get("type")
        type_failed = type_name is not None and not _type_ok(value, type_name)
        if type_failed:
            errors.append("type")
        else:
            size = len(value) if isinstance(value, str) else value
            if "min" in rule and size < rule["min"]:
                errors.append("min")
            if "max" in rule and size > rule["max"]:
                errors.append("max")
            pattern = rule.get("pattern")
            if pattern is not None and not (
                isinstance(value, str) and re.fullmatch(pattern, value)
            ):
                errors.append("pattern")
        if "one_of" in rule and value not in rule["one_of"]:
            errors.append("one_of")
        if errors:
            result[field] = errors
    return result
