# Convert text to a URL slug

- **Language:** python
- **Target file:** `slugify.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def slugify(text: str, *, max_length: int | None = None, separator: str = "-") -> str
```

## Behavior

Lowercase the text, transliterate accented Latin letters to ASCII (use unicodedata NFKD and drop combining marks; additionally map 'ß'->'ss', 'æ'->'ae', 'ø'->'o'), replace every run of non-alphanumeric ASCII characters with a single `separator`, and strip leading/trailing separators. If `max_length` is given, truncate to at most that many chars without leaving a trailing separator.

## Edge cases

- Empty or all-symbol input -> "".
- Digits are kept.
- Truncation happens after slug creation; strip a trailing separator afterwards.
- Non-Latin characters that cannot be transliterated are treated as non-alphanumeric.

## Examples

- "Hello, World!" -> "hello-world"
- "  Crème Brûlée  à la Carte " -> "creme-brulee-a-la-carte"
- "Straße & Co." , separator="_" -> "strasse_co"
- "one two three", max_length=8 -> "one-two"
- "!!!" -> ""
