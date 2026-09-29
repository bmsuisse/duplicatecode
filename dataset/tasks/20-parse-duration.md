# Parse human duration strings

- **Language:** python
- **Target file:** `parse_duration.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def parse_duration(text: str) -> float
```

## Behavior

Parse strings like '1h 30m', '2d', '90s', '1.5h', '500ms', '1w 2d 3h 4m 5s' into total seconds (float). Units: ms, s, m, h, d, w (w=7d). Components separated by optional whitespace, any order, each unit at most once. Case-insensitive. A bare number with no unit is invalid. Leading '-' is not supported.

## Edge cases

- Empty / whitespace -> ValueError.
- Repeated unit ('1h 2h') -> ValueError.
- Unknown unit -> ValueError.
- Compact form '1h30m' is accepted.

## Examples

- "1h 30m" -> 5400.0
- "1h30m" -> 5400.0
- "500ms" -> 0.5
- "1w 1d" -> 691200.0
- "10" -> ValueError
- "1h 2h" -> ValueError
