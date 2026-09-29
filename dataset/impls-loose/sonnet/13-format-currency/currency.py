"""Currency formatting."""

from decimal import ROUND_HALF_UP, Decimal

_SYMBOLS = {"USD": "$", "EUR": "€", "GBP": "£", "CHF": "CHF", "JPY": "¥"}
_ZERO_DECIMAL = {"JPY", "KRW", "VND"}


def format_currency(
    amount: Decimal | float | str,
    currency: str = "USD",
    *,
    thousands_sep: str = ",",
    decimal_sep: str = ".",
    symbol_after: bool = False,
) -> str:
    """Format ``amount`` like ``$1,234.50`` (negative amounts as ``-$1,234.50``)."""
    value = Decimal(str(amount))
    places = 0 if currency.upper() in _ZERO_DECIMAL else 2
    quantum = Decimal(1).scaleb(-places)
    value = value.quantize(quantum, rounding=ROUND_HALF_UP)
    negative = value < 0
    integer, _, fraction = f"{abs(value):f}".partition(".")
    grouped = f"{int(integer):,}".replace(",", thousands_sep)
    number = grouped + (decimal_sep + fraction if places else "")
    symbol = _SYMBOLS.get(currency.upper(), currency.upper())
    if symbol_after:
        text = f"{number} {symbol}"
    elif len(symbol) > 1:
        text = f"{symbol} {number}"
    else:
        text = f"{symbol}{number}"
    return f"-{text}" if negative else text
