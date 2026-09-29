"""Declarative validation of dict rows."""

import re
from collections.abc import Callable, Mapping
from dataclasses import dataclass, field
from typing import Any


@dataclass(frozen=True)
class Rule:
    required: bool = False
    type: type | tuple[type, ...] | None = None
    min_value: float | None = None
    max_value: float | None = None
    min_length: int | None = None
    max_length: int | None = None
    pattern: str | None = None
    choices: tuple[Any, ...] | None = None
    check: Callable[[Any], bool] | None = None
    message: str | None = None  # overrides the check failure message


@dataclass
class ValidationResult:
    errors: dict[str, list[str]] = field(default_factory=dict)

    @property
    def ok(self) -> bool:
        return not self.errors


def validate_row(row: Mapping[str, Any], rules: Mapping[str, Rule]) -> ValidationResult:
    """Validate ``row`` against ``rules`` and collect all failures per field."""
    result = ValidationResult()
    for name, rule in rules.items():
        errors = _check_field(row.get(name), name in row, rule)
        if errors:
            result.errors[name] = errors
    return result


def _check_field(value: Any, present: bool, rule: Rule) -> list[str]:
    if not present or value is None or value == "":
        return ["is required"] if rule.required else []
    errors: list[str] = []
    if rule.type is not None and (
        not isinstance(value, rule.type)
        or (isinstance(value, bool) and bool not in _as_tuple(rule.type))
    ):
        return ["has the wrong type"]
    if isinstance(value, int | float) and not isinstance(value, bool):
        if rule.min_value is not None and value < rule.min_value:
            errors.append(f"must be >= {rule.min_value}")
        if rule.max_value is not None and value > rule.max_value:
            errors.append(f"must be <= {rule.max_value}")
    if isinstance(value, str | list | tuple | dict):
        if rule.min_length is not None and len(value) < rule.min_length:
            errors.append(f"must have length >= {rule.min_length}")
        if rule.max_length is not None and len(value) > rule.max_length:
            errors.append(f"must have length <= {rule.max_length}")
    if rule.pattern is not None and not (
        isinstance(value, str) and re.fullmatch(rule.pattern, value)
    ):
        errors.append("does not match the required pattern")
    if rule.choices is not None and value not in rule.choices:
        errors.append(f"must be one of {list(rule.choices)}")
    if rule.check is not None and not rule.check(value):
        errors.append(rule.message or "failed validation")
    return errors


def _as_tuple(t: type | tuple[type, ...]) -> tuple[type, ...]:
    return t if isinstance(t, tuple) else (t,)
