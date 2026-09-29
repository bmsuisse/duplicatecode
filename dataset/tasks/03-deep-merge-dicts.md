# Recursive dictionary deep merge

- **Language:** python
- **Target file:** `deep_merge_dicts.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def deep_merge(base: dict[str, Any], override: dict[str, Any], *, list_strategy: Literal["replace", "concat", "unique"] = "replace") -> dict[str, Any]
```

## Behavior

Return a new dict combining `base` and `override` without mutating either. Keys only in one side are copied. When both values are dicts, merge recursively. When both are lists, apply `list_strategy`: replace (override wins), concat (base + override), unique (concat then drop duplicates preserving first occurrence order; items are hashable or compared with ==). Otherwise override wins, including when override value is None.

## Edge cases

- Result must not share nested mutable containers with inputs (deep copy).
- Type mismatch (dict vs scalar) -> override wins.
- Empty dicts are valid inputs.

## Examples

- {"a":1,"b":{"x":1}} + {"b":{"y":2},"c":3} -> {"a":1,"b":{"x":1,"y":2},"c":3}
- {"l":[1,2]} + {"l":[2,3]}, concat -> {"l":[1,2,2,3]}
- same, unique -> {"l":[1,2,3]}
- {"a":{"b":1}} + {"a":None} -> {"a":None}
- {"a":{"b":1}} + {"a":5} -> {"a":5}
