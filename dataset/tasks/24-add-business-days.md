# Add business days to a date

- **Language:** python
- **Target file:** `add_business_days.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def add_business_days(start: date, days: int, holidays: Collection[date] = ()) -> date
```

## Behavior

Business days are Monday-Friday not in `holidays`. Move `days` business days from `start` (negative moves backwards). days == 0 returns start unchanged even if start is a weekend/holiday. The start date itself is never counted; each step moves to the next business day in that direction.

## Edge cases

- Weekend start with positive days: first step lands on the next business day.
- Holidays on weekends are irrelevant.

## Examples

- Fri 2024-03-01 + 1 -> Mon 2024-03-04
- Mon 2024-03-04 - 1 -> Fri 2024-03-01
- Wed 2024-03-06 + 5 -> Wed 2024-03-13
- Fri 2024-03-01 + 1 with holidays {2024-03-04} -> Tue 2024-03-05
- Sat 2024-03-02 + 0 -> 2024-03-02
