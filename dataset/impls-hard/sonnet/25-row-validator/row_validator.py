"""Declarative validation of data rows."""

import re
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class FieldRule:
    """Validation rules for one field. Unset options are not checked."""

    required: bool = False
    min_length: int | None = None
    max_length: int | None = None
    min_value: float | None = None
    max_value: float | None = None
    pattern: str | None = None
    allowed: Sequence[Any] | None = None
    custom: Callable[[Any], str | None] | None = None


def _is_empty(value: Any) -> bool:
    return value is None or (isinstance(value, str) and value.strip() == "")


def _check_field(value: Any, rule: FieldRule) -> list[str]:
    errors: list[str] = []
    if _is_empty(value):
        if rule.required:
            errors.append("value is required")
        return errors
    if rule.min_length is not None or rule.max_length is not None:
        length = len(str(value))
        if rule.min_length is not None and length < rule.min_length:
            errors.append(f"must have at least {rule.min_length} characters")
        if rule.max_length is not None and length > rule.max_length:
            errors.append(f"must have at most {rule.max_length} characters")
    if rule.min_value is not None or rule.max_value is not None:
        try:
            number = float(value)
        except (TypeError, ValueError):
            errors.append("must be a number")
        else:
            if rule.min_value is not None and number < rule.min_value:
                errors.append(f"must be at least {rule.min_value}")
            if rule.max_value is not None and number > rule.max_value:
                errors.append(f"must be at most {rule.max_value}")
    if rule.pattern is not None and not re.fullmatch(rule.pattern, str(value)):
        errors.append("has an invalid format")
    if rule.allowed is not None and value not in rule.allowed:
        errors.append("is not an allowed value")
    if rule.custom is not None and (message := rule.custom(value)):
        errors.append(message)
    return errors


def validate_row(row: Mapping[str, Any], rules: Mapping[str, FieldRule]) -> dict[str, list[str]]:
    """Return a mapping of field name to error messages; empty if the row is valid."""
    result: dict[str, list[str]] = {}
    for name, rule in rules.items():
        errors = _check_field(row.get(name), rule)
        if errors:
            result[name] = errors
    return result
