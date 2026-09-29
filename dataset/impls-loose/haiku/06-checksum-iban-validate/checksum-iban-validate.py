def validate_iban(iban: str) -> bool:
    """Validate IBAN using mod-97 checksum."""
    iban = iban.replace(" ", "").replace("-", "").upper()

    # Move first 4 chars to end
    rearranged = iban[4:] + iban[:4]

    # Replace letters with numbers (A=10, B=11, etc.)
    numeric = ""
    for char in rearranged:
        if char.isdigit():
            numeric += char
        else:
            numeric += str(ord(char) - ord("A") + 10)

    # Check mod 97
    return int(numeric) % 97 == 1
