# Diff two flat records

- **Language:** python
- **Target file:** `diff_records.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def diff_records(old: Mapping[str, Any], new: Mapping[str, Any], *, ignore: Collection[str] = ()) -> dict[str, dict[str, Any]]
```

## Behavior

Return {"added": {k: v}, "removed": {k: v}, "changed": {k: {"old": a, "new": b}}}. Keys in `ignore` are skipped everywhere. A key present in both with equal values (==) is unchanged. A key present with value None is still 'present'. Each of the three sections is always present (possibly empty). Section key order follows first appearance in old, then new.

## Edge cases

- Both empty -> three empty dicts.
- 1 vs 1.0 are equal (Python ==).
- Nested values compared with == only (no recursion).

## Examples

- old={"a":1,"b":2}, new={"a":1,"b":3,"c":4} -> added {c:4}, removed {}, changed {b:{old:2,new:3}}
- old={"a":1}, new={} -> removed {a:1}
- old={"a":None}, new={"a":0} -> changed {a:{old:None,new:0}}
- ignore=["updated_at"] with differing updated_at only -> all empty
