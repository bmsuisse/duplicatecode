import re


def parse_duration(text: str) -> float:
    text = text.strip()
    if not text:
        raise ValueError("Empty duration string")

    unit_multipliers = {
        "ms": 0.001,
        "s": 1.0,
        "m": 60.0,
        "h": 3600.0,
        "d": 86400.0,
        "w": 604800.0,
    }

    # Find all components: number + unit
    pattern = r"(\d+\.?\d*)(ms|[smhdw])"
    matches = re.findall(pattern, text, re.IGNORECASE)

    if not matches:
        raise ValueError("No valid duration components")

    # Check if all input is consumed
    consumed = re.sub(pattern, "", text, flags=re.IGNORECASE).strip()
    if consumed:
        raise ValueError(f"Invalid duration format: unrecognized '{consumed}'")

    total_seconds = 0.0
    seen_units = set()

    for value_str, unit in matches:
        unit_lower = unit.lower()

        if unit_lower in seen_units:
            raise ValueError(f"Duplicate unit: {unit}")

        seen_units.add(unit_lower)

        if unit_lower not in unit_multipliers:
            raise ValueError(f"Unknown unit: {unit}")

        value = float(value_str)
        total_seconds += value * unit_multipliers[unit_lower]

    return total_seconds
