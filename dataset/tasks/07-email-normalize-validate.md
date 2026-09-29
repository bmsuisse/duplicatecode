# Email validation and normalisation

- **Language:** python
- **Target file:** `email_normalize_validate.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def normalize_email(value: str) -> str | None
```

## Behavior

Trim whitespace and lowercase the domain part. Return None if invalid. Valid: exactly one '@'; local part 1-64 chars of [A-Za-z0-9.!#$%&'*+/=?^_`{|}~-], not starting/ending with '.', no '..'; domain has at least one dot, labels 1-63 chars of [A-Za-z0-9-] not starting/ending with '-', last label at least 2 letters. Local part case is preserved.

## Edge cases

- Whitespace inside the address -> None.
- Empty string -> None.
- Total length > 254 -> None.

## Examples

- "  John.Doe@Example.COM " -> "John.Doe@example.com"
- "a@b" -> None
- "a..b@x.io" -> None
- "user+tag@sub.domain.org" -> "user+tag@sub.domain.org"
- "@x.com" -> None
