def format_currency(amount: float, currency: str = "USD") -> str:
    """Format a monetary amount with thousands separators."""
    # Format with 2 decimal places and thousands separator
    formatted = f"{amount:,.2f}"

    # Add currency symbol
    symbols = {
        "USD": "$",
        "EUR": "€",
        "GBP": "£",
        "CHF": "CHF ",
        "JPY": "¥",
    }

    symbol = symbols.get(currency, f"{currency} ")
    return f"{symbol}{formatted}"
