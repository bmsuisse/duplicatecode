# Currency formatting with thousands separators

- **Language:** python
- **Target file:** `format_currency.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def format_currency(amount: Decimal | int | float | str, currency: str = "CHF", *, thousands: str = "'", decimal: str = ".", symbol_position: Literal["prefix", "suffix"] = "prefix", negative_style: Literal["minus", "parens"] = "minus") -> str
```

## Behavior

Round half-up (ROUND_HALF_UP on Decimal built from str(amount)) to 2 decimals, group integer part in thousands with `thousands`, join with `decimal`. Currency code is uppercased and separated from the number by one space, placed per `symbol_position`. Negative: minus -> '-' before the number (before the code when prefix: '-CHF 5.00'); parens -> '(CHF 5.00)'. Negative zero after rounding prints as positive.

## Edge cases

- Invalid amount string -> ValueError.
- Large values (>= 1e9) still grouped.
- -0.004 -> 0.00 without a sign.

## Examples

- format_currency(1234567.891) -> "CHF 1'234'567.89"
- format_currency("0.005", "eur", thousands=",") -> "EUR 0.01"
- format_currency(-1500, "USD", thousands=",", negative_style="parens") -> "(USD 1,500.00)"
- format_currency(12, "EUR", decimal=",", symbol_position="suffix") -> "12,00 EUR"
- format_currency(-0.004) -> "CHF 0.00"
