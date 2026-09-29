def validate_iban(iban: str) -> bool:
    """Validate IBAN using mod 97 checksum."""
    iban = iban.replace(" ", "").replace("-", "").upper()

    if len(iban) < 15 or len(iban) > 34:
        return False

    # Move first 4 characters to end
    rearranged = iban[4:] + iban[:4]

    # Replace letters with numbers (A=10, B=11, ..., Z=35)
    numeric_str = ""
    for char in rearranged:
        if char.isdigit():
            numeric_str += char
        elif char.isalpha():
            numeric_str += str(ord(char) - ord("A") + 10)
        else:
            return False

    # Check mod 97
    return int(numeric_str) % 97 == 1
