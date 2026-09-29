# Deep merge plain objects

- **Language:** typescript
- **Target file:** `deep-merge-objects.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export function deepMerge<T extends Record<string, unknown>, U extends Record<string, unknown>>(base: T, override: U, opts?: { arrays?: 'replace' | 'concat' | 'unique' }): T & U
```

## Behavior

Returns a new object; inputs are not mutated and result does not share nested objects/arrays with inputs. Plain objects merge recursively; arrays follow `arrays` (default replace; concat = base then override; unique = concat with strict-equality dedupe keeping first occurrences). Any other case: override wins, including explicit null. Keys whose override value is `undefined` are ignored (base value kept). Keys '__proto__', 'constructor' and 'prototype' are skipped.

## Edge cases

- Date/Map/Set/class instances count as non-plain values -> override wins by reference.
- Type mismatch object vs primitive -> override wins.

## Examples

- {a:1,b:{x:1}} + {b:{y:2},c:3} -> {a:1,b:{x:1,y:2},c:3}
- {l:[1,2]} + {l:[2,3]}, concat -> {l:[1,2,2,3]}; unique -> {l:[1,2,3]}
- {a:1} + {a:undefined} -> {a:1}
- {a:{b:1}} + {a:null} -> {a:null}
- JSON.parse('{"__proto__":{"x":1}}') as override -> {} polluted nothing
