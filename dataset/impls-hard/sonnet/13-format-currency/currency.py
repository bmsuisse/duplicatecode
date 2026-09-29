"""Format monetary amounts for documents."""

from decimal import ROUND_HALF_UP, Decimal

_SEPARATORS = {
    "en": (",", "."),
    "de-ch": ("’", "."),
    "de": (".", ","),
}


def format_currency(
    amount: Decimal | int | str,
    currency: str,
    *,
    locale: str = "en",
) -> str:
    """Return e.g. ``CHF 1,234.50`` (en) or ``CHF 1’234.50`` (de-ch).

    The amount is rounded half-up to two decimals. Floats are rejected to avoid
    binary rounding surprises; pass ``Decimal``, ``int`` or ``str``.
    """
    if isinstance(amount, float):
        raise TypeError("use Decimal, int or str for monetary amounts")
    if locale not in _SEPARATORS:
        raise ValueError(f"unsupported locale: {locale!r}")
    code = currency.strip().upper()
    if len(code) != 3 or not code.isalpha():
        raise ValueError(f"invalid currency code: {currency!r}")
    value = Decimal(amount).quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)
    negative = value < 0
    whole, _, cents = f"{abs(value):f}".partition(".")
    thousands, decimal = _SEPARATORS[locale]
    groups: list[str] = []
    while len(whole) > 3:
        groups.insert(0, whole[-3:])
        whole = whole[:-3]
    groups.insert(0, whole)
    text = f"{code} {'-' if negative else ''}{thousands.join(groups)}{decimal}{cents}"
    return text
