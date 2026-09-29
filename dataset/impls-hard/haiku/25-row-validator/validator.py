from collections.abc import Callable
from typing import Any


class RowValidator:
    """Declarative row validation."""

    def __init__(self):
        self.rules: dict[str, list[Callable[[Any], bool]]] = {}

    def add_rule(self, field: str, rule: Callable[[Any], bool]) -> None:
        """Add validation rule for field."""
        if field not in self.rules:
            self.rules[field] = []
        self.rules[field].append(rule)

    def validate(self, row: dict[str, Any]) -> tuple[bool, dict[str, list[str]]]:
        """Validate row returning (is_valid, errors)."""
        errors: dict[str, list[str]] = {}

        for field, rule_list in self.rules.items():
            value = row.get(field)
            for rule in rule_list:
                if not rule(value):
                    if field not in errors:
                        errors[field] = []
                    errors[field].append(f"Validation failed for {field}")

        return len(errors) == 0, errors
