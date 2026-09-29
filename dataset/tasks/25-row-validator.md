# Declarative row validation

- **Language:** python
- **Target file:** `row_validator.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def validate_row(row: Mapping[str, Any], rules: Mapping[str, Mapping[str, Any]]) -> dict[str, list[str]]
```

## Behavior

`rules` maps field -> dict with optional keys: required (bool), type ("str"|"int"|"float"|"bool"), min/max (numbers for int/float, lengths for str), pattern (regex full match for str), one_of (list). Return field -> list of error codes, only for fields with errors, codes emitted in this order: 'required', 'type', 'min', 'max', 'pattern', 'one_of'. Missing/None/"" counts as empty: if empty and required -> ['required'] only; if empty and not required -> no checks. bool is not accepted for int/float. int accepted for float. Fields not in rules are ignored.

## Edge cases

- Errors after 'type' failure: skip min/max/pattern (only 'one_of' still checked).
- min/max on str compare len().

## Examples

- row {name:"",age:17}, rules {name:{required:True}, age:{type:"int",min:18}} -> {name:["required"], age:["min"]}
- {age:"x"} with age type int -> {age:["type"]}
- {code:"ab1"} pattern [a-z]+ -> {code:["pattern"]}
- {flag:True} type int -> {flag:["type"]}
- all OK -> {}
