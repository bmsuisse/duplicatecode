# Normalise organisation names for duplicate matching

- **Language:** python
- **Target file:** `company_name_normalizer.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def normalize_org_name(name: str) -> str
def org_names_match(a: str, b: str) -> bool
```

## Behavior

`normalize_org_name`: casefold; transliterate accents (NFKD, drop combining); replace '&' with ' and '; drop punctuation (anything not alphanumeric/space); collapse whitespace; then remove trailing legal-form tokens repeatedly from this set: ag, gmbh, sa, sarl, ltd, limited, inc, llc, co, corp, plc, bv, srl, spa (only if at least one other token would remain). `org_names_match`: True if normalized forms are equal and non-empty.

## Edge cases

- Legal forms in the middle are kept.
- 'Ltd' alone stays 'ltd'.
- Empty -> "".

## Examples

- "Müller & Söhne GmbH" -> "muller and sohne"
- "ACME, Inc." -> "acme"
- "Foo Co. Ltd" -> "foo"
- "Ltd" -> "ltd"
- org_names_match("Acme AG", "acme") -> True
