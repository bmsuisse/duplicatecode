# Async map with concurrency limit

- **Language:** typescript
- **Target file:** `concurrency-limited-map.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export async function mapLimit<T, R>(items: readonly T[], limit: number, fn: (item: T, index: number) => Promise<R>): Promise<R[]>
```

## Behavior

Applies async `fn` to all items with at most `limit` running simultaneously and returns results in input order. Starts a new task as soon as one finishes (worker pool, not batches). If any call rejects, the returned promise rejects with that error and no further tasks are started (already running ones may finish). limit is clamped to items.length.

## Edge cases

- limit not a positive integer -> rejects with RangeError.
- Empty items -> [].
- Non-array-mutating.

## Examples

- items [1,2,3], limit 2, fn doubles -> [2,4,6]
- 5 items with limit 2 -> observed max concurrency 2
- fn rejects for item 2 of 5, limit 1 -> rejects; items 3-5 never started
- [] -> []
