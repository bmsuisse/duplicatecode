# Date period labels

- **Language:** python
- **Target file:** `format_period_label.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def format_period(start: date, end: date, *, locale: Literal["en", "de"] = "en") -> str
```

## Behavior

Return a compact label for an inclusive date range. Month names: en Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec; de Jan Feb Mär Apr Mai Jun Jul Aug Sep Okt Nov Dez. Rules in order: start > end -> ValueError. Exactly a whole calendar month -> "Mar 2024". Exactly a whole calendar quarter -> "Q1 2024". Exactly a whole calendar year -> "2024". Same day -> "5 Mar 2024". Same month -> "5-9 Mar 2024" (en dash not used, plain hyphen). Same year -> "28 Feb - 3 Apr 2024". Otherwise "28 Dec 2023 - 3 Jan 2024".

## Edge cases

- Leap-year February 1-29 is a whole month.
- German uses the same layout with German month names.
- A range spanning two full months is NOT a quarter and falls to the same-year rule.

## Examples

- 2024-03-01..2024-03-31 -> "Mar 2024"
- 2024-01-01..2024-03-31 -> "Q1 2024"
- 2024-03-05..2024-03-09 -> "5-9 Mar 2024"
- 2024-02-28..2024-04-03 -> "28 Feb - 3 Apr 2024"
- 2024-03-01..2024-03-31, locale=de -> "Mär 2024"
