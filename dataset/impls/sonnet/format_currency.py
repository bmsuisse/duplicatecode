"""Currency formatting with thousands separators."""

from decimal import ROUND_HALF_UP, Decimal, InvalidOperation
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
    """Format ``amount`` with two decimals, grouping and a currency code."""
    try:
        value = Decimal(str(amount))
    except InvalidOperation:
        raise ValueError(f"invalid amount: {amount!r}") from None
    if not value.is_finite():
        raise ValueError(f"invalid amount: {amount!r}")
    rounded = value.quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)
    negative = rounded < 0
    whole, _, fraction = f"{abs(rounded):f}".partition(".")
    groups = []
    while len(whole) > 3:
        groups.insert(0, whole[-3:])
        whole = whole[:-3]
    groups.insert(0, whole)
    number = thousands.join(groups) + decimal + fraction
    code = currency.upper()
    text = f"{code} {number}" if symbol_position == "prefix" else f"{number} {code}"
    if not negative:
        return text
    return f"({text})" if negative_style == "parens" else f"-{text}"
