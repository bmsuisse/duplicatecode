# Map CSV rows to typed records

- **Language:** python
- **Target file:** `csv_row_mapper.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def map_rows(rows: Iterable[Sequence[str]], mapping: Mapping[str, tuple[str, Callable[[str], Any]]], *, has_header: bool = True) -> tuple[list[dict[str, Any]], list[str]]
```

## Behavior

`mapping` maps a header name (matched case-insensitively after strip) to (output_key, converter). First row is header when has_header. For each data row build a dict of output_key -> converter(cell.strip()). Columns not in mapping are ignored. Blank cells (after strip) become None without calling the converter. If a converter raises ValueError, the row is skipped and an error string `row {n}: {output_key}: {message}` (n = 1-based data row number, header excluded) is appended. Fully empty rows are skipped silently (they still count toward n). Returns (records, errors). Missing header column present in mapping -> ValueError.

## Edge cases

- Short rows: missing trailing cells are None.
- Duplicate header names: last wins.
- has_header False: mapping keys are 0-based column indexes as strings ("0", "1").

## Examples

- rows=[["Name","Age"],["Ann","31"],["Bob",""]], mapping={"name":("name",str),"age":("age",int)} -> ([{name:"Ann",age:31},{name:"Bob",age:None}], [])
- [["a"],["x"]] with converter int on a -> ([], ["row 1: a: invalid literal for int() with base 10: 'x'"])
- header lacks a mapped column -> ValueError
