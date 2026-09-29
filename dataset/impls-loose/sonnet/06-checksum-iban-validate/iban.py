"""IBAN validation using the ISO 7064 mod-97 checksum."""

import re

_FORMAT = re.compile(r"^[A-Z]{2}\d{2}[A-Z0-9]{1,30}$")


def is_valid_iban(value: str) -> bool:
    """Return True if ``value`` is a well-formed IBAN with a valid mod-97 check."""
    iban = re.sub(r"[\s-]", "", value).upper()
    if not 5 <= len(iban) <= 34 or not _FORMAT.match(iban):
        return False
    rearranged = iban[4:] + iban[:4]
    # Letters map to 10..35; digits stay as they are.
    digits = "".join(str(int(ch, 36)) for ch in rearranged)
    return int(digits) % 97 == 1
