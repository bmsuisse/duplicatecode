# Validate IBAN-style account numbers (mod 97)

- **Language:** python
- **Target file:** `checksum_iban_validate.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def validate_iban(value: str) -> bool
def format_iban(value: str) -> str
```

## Behavior

`validate_iban`: remove spaces and hyphens, uppercase; must match ^[A-Z]{2}[0-9]{2}[A-Z0-9]{11,30}$ ; move first 4 chars to the end; replace letters A=10..Z=35; the resulting integer mod 97 must equal 1. Do not check country-specific lengths. `format_iban`: return normalized value in groups of 4 separated by single spaces; raise ValueError if not valid.

## Edge cases

- Non-string-like garbage / empty -> False.
- Lowercase and spaces accepted.
- Use integer arithmetic on the full number (Python ints) or piecewise mod; result must be identical.

## Examples

- "DE89 3704 0044 0532 0130 00" -> True
- "gb82west12345698765432" -> True
- "DE89370400440532013001" -> False
- "12345" -> False
- format_iban("ch9300762011623852957") -> "CH93 0076 2011 6238 5295 7"
