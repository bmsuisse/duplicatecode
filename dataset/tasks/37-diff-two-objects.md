# Diff two flat objects

- **Language:** typescript
- **Target file:** `diff-two-objects.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export type Diff = { added: Record<string, unknown>; removed: Record<string, unknown>; changed: Record<string, { from: unknown; to: unknown }> }
export function diffObjects(before: Record<string, unknown>, after: Record<string, unknown>, opts?: { ignore?: readonly string[] }): Diff
```

## Behavior

Compare own enumerable keys. added: keys only in `after`; removed: keys only in `before`; changed: keys in both whose values differ. Equality is Object.is-like for primitives, and JSON-structural (deep equal for plain objects/arrays, Dates by getTime) for containers. Keys in `ignore` skipped. All three sections always present. A key present with value undefined counts as present.

## Edge cases

- NaN equals NaN.
- 0 and -0 are considered equal.
- Key order in results follows before then after.

## Examples

- {a:1,b:2} vs {a:1,b:3,c:4} -> added {c:4}, removed {}, changed {b:{from:2,to:3}}
- {a:[1,2]} vs {a:[1,2]} -> all empty
- {a:{x:1}} vs {a:{x:2}} -> changed {a:{from:{x:1},to:{x:2}}}
- ignore ['updatedAt'] with only updatedAt differing -> all empty
- {a:1} vs {} -> removed {a:1}
