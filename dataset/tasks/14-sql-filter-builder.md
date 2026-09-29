# Parameterised SQL WHERE clause builder

- **Language:** python
- **Target file:** `sql_filter_builder.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def build_where(filters: Sequence[Mapping[str, Any]], *, placeholder: str = "?") -> tuple[str, list[Any]]
```

## Behavior

Each filter is {"field": str, "op": str, "value": Any}. Supported ops: eq, ne, gt, gte, lt, lte, like (value used as-is), in, not_in, is_null (value bool: True -> IS NULL, False -> IS NOT NULL), between (value is 2-item list). Return ("WHERE a = ? AND b IN (?, ?)", params) joined with AND; empty filters -> ("", []). Field names must match ^[A-Za-z_][A-Za-z0-9_.]*$ else ValueError; unknown op -> ValueError. An `in` with empty list yields the constant `1 = 0`; `not_in` with empty list is skipped. eq with value None renders `field IS NULL` (no param), ne with None renders `IS NOT NULL`.

## Edge cases

- Operators map: eq '=', ne '<>', gt '>', gte '>=', lt '<', lte '<='.
- between with wrong length -> ValueError.
- Params ordered by appearance.

## Examples

- [{"field":"age","op":"gte","value":18},{"field":"city","op":"in","value":["A","B"]}] -> ("WHERE age >= ? AND city IN (?, ?)", [18,"A","B"])
- [{"field":"x","op":"eq","value":None}] -> ("WHERE x IS NULL", [])
- [{"field":"n","op":"between","value":[1,5]}] -> ("WHERE n BETWEEN ? AND ?", [1,5])
- [{"field":"a;drop","op":"eq","value":1}] -> ValueError
- [] -> ("", [])
