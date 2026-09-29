# Query string builder

- **Language:** typescript
- **Target file:** `query-string-builder.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export function buildQuery(params: Record<string, unknown>, opts?: { arrayFormat?: 'repeat' | 'comma' | 'brackets'; prefix?: boolean }): string
```

## Behavior

Builds a URL query string. Skip undefined and null values. Booleans -> 'true'/'false'. Date -> toISOString(). Arrays: repeat -> k=a&k=b; comma -> k=a,b (commas literal, elements encoded); brackets -> k[]=a&k[]=b (brackets literal). Empty arrays skipped. Keys/values encoded with encodeURIComponent. Insertion order preserved. With prefix:true a non-empty result starts with '?'; empty result is always ''.

## Edge cases

- Empty string value -> `k=`.
- Numbers via String().
- Nested plain objects are not supported: throw TypeError.

## Examples

- {q:'a b', page:2} -> 'q=a%20b&page=2'
- {t:['x','y']} -> 't=x&t=y'; comma -> 't=x,y'; brackets -> 't[]=x&t[]=y'
- {a:null, b:true, c:''} -> 'b=true&c='
- {a:1}, {prefix:true} -> '?a=1'
- {} , {prefix:true} -> ''
