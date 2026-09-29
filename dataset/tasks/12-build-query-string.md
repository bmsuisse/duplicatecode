# Build a URL query string

- **Language:** python
- **Target file:** `build_query_string.py`
- **Constraints:** Standard library only; no third-party packages. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```python
def build_query_string(params: Mapping[str, Any], *, array_format: Literal["repeat", "comma", "brackets"] = "repeat") -> str
```

## Behavior

Return a query string WITHOUT leading '?'. Skip None values. Booleans render as 'true'/'false'. Lists/tuples: repeat -> k=a&k=b ; comma -> k=a,b (commas literal, elements percent-encoded) ; brackets -> k[]=a&k[]=b (brackets literal). Empty lists are skipped. Keys and values are percent-encoded per RFC 3986 unreserved set (urllib.parse.quote with safe=''). Preserve mapping order. Numbers use str().

## Edge cases

- Empty mapping -> "".
- Space -> %20 (not +).
- Empty string value renders as `k=`.

## Examples

- {"q":"a b","page":2} -> "q=a%20b&page=2"
- {"t":["x","y"]} -> "t=x&t=y"
- {"t":["x","y"]}, comma -> "t=x,y"
- {"t":["x","y"]}, brackets -> "t[]=x&t[]=y"
- {"a":None,"b":True,"c":""} -> "b=true&c="
