# Mask sensitive values in nested data

- **Language:** python
- **Target file:** `mask_sensitive_values.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def mask_sensitive(data: Any, *, keys: Collection[str] = ("password", "token", "secret", "authorization"), mask: str = "***") -> Any
```

## Behavior

Return a deep copy of nested dicts/lists/tuples where any dict value whose key (case-insensitive, substring match against any entry in `keys`) matches is replaced by `mask`. Recurse through dicts and lists; tuples stay tuples. Non-container leaves returned unchanged. Do not mutate input. A matched key's value is masked even if it is a container.

## Edge cases

- Non-string keys are never matched but their values are still recursed.
- None value under sensitive key becomes mask too.

## Examples

- {"user":"a","Password":"x"} -> {"user":"a","Password":"***"}
- {"items":[{"api_token":"t","n":1}]} -> {"items":[{"api_token":"***","n":1}]}
- {"secret":{"a":1}} -> {"secret":"***"}
- 5 -> 5
