def format_currency(amount: float, currency: str = "USD", decimals: int = 2) -> str:
    """Format amount as currency with thousands separator."""
    symbols = {"USD": "$", "EUR": "€", "GBP": "£", "CHF": "CHF"}
    symbol = symbols.get(currency, currency)

    # Format with thousands separator
    formatted = f"{amount:,.{decimals}f}"
    return f"{symbol} {formatted}"
