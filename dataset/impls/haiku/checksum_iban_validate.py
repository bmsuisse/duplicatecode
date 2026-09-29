import re


def validate_iban(value: str) -> bool:
    # Remove spaces and hyphens, uppercase
    iban = value.replace(" ", "").replace("-", "").upper()

    # Check format
    if not re.match(r"^[A-Z]{2}[0-9]{2}[A-Z0-9]{11,30}$", iban):
        return False

    # Move first 4 chars to the end
    moved = iban[4:] + iban[:4]

    # Replace letters with numbers (A=10, B=11, ..., Z=35)
    numeric_str = ""
    for char in moved:
        if char.isdigit():
            numeric_str += char
        else:
            numeric_str += str(ord(char) - ord("A") + 10)

    # Check mod 97
    return int(numeric_str) % 97 == 1


def format_iban(value: str) -> str:
    if not validate_iban(value):
        raise ValueError("Invalid IBAN")

    # Normalize and format in groups of 4
    iban = value.replace(" ", "").replace("-", "").upper()
    return " ".join(iban[i : i + 4] for i in range(0, len(iban), 4))
