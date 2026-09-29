# Multi-key sort comparator

- **Language:** typescript
- **Target file:** `multi-key-sorter.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export type SortSpec<T> = { key: keyof T; dir?: 'asc' | 'desc'; nulls?: 'first' | 'last' }
export function sortBy<T>(items: readonly T[], specs: readonly SortSpec<T>[]): T[]
```

## Behavior

Return a new sorted array (stable). Compare by each spec in order until non-zero. Numbers compare numerically, strings with localeCompare(undefined, {sensitivity:'base', numeric:true}), Dates by time, booleans false<true. null/undefined are placed last by default (nulls:'last') or first regardless of dir. dir 'desc' inverts non-null comparisons only. Input not mutated.

## Edge cases

- Empty specs -> shallow copy in original order.
- Mixed types on same key: compare String() forms.

## Examples

- [{n:'b',a:2},{n:'a',a:2},{n:'c',a:1}] by a asc, n asc -> c, a, b
- values [3,null,1,2] by v desc -> [3,2,1,null]
- same, nulls:'first' -> [null,3,2,1]
- names ['item10','item2'] asc -> ['item2','item10']
