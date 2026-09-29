# Chunk an array

- **Language:** typescript
- **Target file:** `chunk-array.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export function chunk<T>(items: readonly T[], size: number, opts?: { dropRemainder?: boolean }): T[][]
```

## Behavior

Split into consecutive arrays of length `size`. Last chunk may be shorter unless dropRemainder is true, in which case incomplete last chunk is dropped.

## Edge cases

- size not a positive integer -> throws RangeError.
- Empty input -> [].
- Returned chunks are new arrays; input not mutated.

## Examples

- chunk([1,2,3,4,5], 2) -> [[1,2],[3,4],[5]]
- chunk([1,2,3,4,5], 2, {dropRemainder:true}) -> [[1,2],[3,4]]
- chunk([], 3) -> []
- chunk([1], 5) -> [[1]]
- chunk([1], 0) -> throws
