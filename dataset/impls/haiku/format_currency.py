from decimal import ROUND_HALF_UP, Decimal
from typing import Literal


def format_currency(
    amount: Decimal | float | str,
    currency: str = "CHF",
    *,
    thousands: str = "'",
    decimal: str = ".",
    symbol_position: Literal["prefix", "suffix"] = "prefix",
    negative_style: Literal["minus", "parens"] = "minus",
) -> str:
    # Convert to Decimal and round half-up to 2 decimals
    d = Decimal(str(amount))
    d = d.quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)

    # Check if negative
    is_negative = d < 0
    d = abs(d)

    # Split into integer and decimal parts
    d_str = str(d)
    if "." in d_str:
        int_part, dec_part = d_str.split(".")
    else:
        int_part = d_str
        dec_part = "00"

    # Ensure decimal part is exactly 2 digits
    dec_part = dec_part.ljust(2, "0")[:2]

    # Add thousands separators to integer part
    int_part_with_sep = ""
    for i, digit in enumerate(reversed(int_part)):
        if i > 0 and i % 3 == 0:
            int_part_with_sep = thousands + int_part_with_sep
        int_part_with_sep = digit + int_part_with_sep

    # Build the number string
    number_str = int_part_with_sep + decimal + dec_part

    # Add currency code
    currency_upper = currency.upper()
    if symbol_position == "prefix":
        result = f"{currency_upper} {number_str}"
    else:
        result = f"{number_str} {currency_upper}"

    # Handle negative
    if is_negative:
        if negative_style == "minus":
            result = f"-{result}"
        elif negative_style == "parens":
            result = f"({result})"

    return result
