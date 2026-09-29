# Master-data record merge with survivorship rules

- **Language:** python
- **Target file:** `record_survivorship_merge.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def merge_records(records: Sequence[Mapping[str, Any]], rules: Mapping[str, Literal["most_recent", "most_complete", "first_non_null", "longest"]], *, updated_key: str = "updated_at") -> dict[str, Any]
```

## Behavior

Merge duplicate records into one golden record. Each record has fields plus a sortable `updated_key` (ISO string or datetime). For each field in the union of keys (excluding updated_key; order of first appearance) apply the rule (default first_non_null when no rule): most_recent = value from the record with the greatest updated_key that has a non-empty value (None and "" are empty); first_non_null = first non-empty value in input order; longest = non-empty value with greatest len(str(v)) (ties -> earliest in input); most_complete = value from the record that has the most non-empty fields overall (ties -> most recent) if that record's value is non-empty, else fall back to first_non_null. Output includes updated_key = max of inputs. Fields with no non-empty value -> None.

## Edge cases

- Empty records -> ValueError.
- Single record returns a copy with empties normalised to None.
- Rules for unknown field names are ignored.

## Examples

- [{name:"Acme",phone:None,updated_at:"2024-01-01"},{name:"Acme Ltd",phone:"123",updated_at:"2024-02-01"}], rules {name:"longest",phone:"most_recent"} -> {name:"Acme Ltd",phone:"123",updated_at:"2024-02-01"}
- same, rules {name:"first_non_null"} -> name "Acme"
- [{a:"",u..},{a:None,..}] -> a None
