# Phone number normalisation to E.164-like form

- **Language:** python
- **Target file:** `phone_normalize.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def normalize_phone(raw: str, default_country_code: str = "41") -> str | None
```

## Behavior

Strip spaces, dots, hyphens, parentheses and slashes. Handle prefixes: leading '+' keeps the digits; leading '00' is treated as '+'; a single leading '0' is a national number -> replace the 0 with `default_country_code`. Also drop a '(0)' trunk marker directly following a country code (e.g. '+41 (0) 44 ...'). Return '+' followed by digits. Return None if the resulting digit count is not between 8 and 15, or if input contains letters or other characters.

## Edge cases

- Input without any recognised prefix (no +, 00, 0) -> None.
- Empty -> None.
- Only the listed separators are allowed.

## Examples

- "044 123 45 67" -> "+41441234567"
- "+41 (0) 44 123 45 67" -> "+41441234567"
- "0049 30 1234567" -> "+49301234567"
- "+1 (555) 010-9999" -> "+15550109999"
- "12abc" -> None
