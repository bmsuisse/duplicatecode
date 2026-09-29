# Group and count items by key

- **Language:** typescript
- **Target file:** `group-items-by.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export function groupBy<T, K extends string | number>(items: readonly T[], keyFn: (item: T, index: number) => K): Map<K, T[]>
export function countBy<T, K extends string | number>(items: readonly T[], keyFn: (item: T) => K): Map<K, number>
```

## Behavior

groupBy returns a Map keyed by keyFn result, keys in first-seen order, items in input order. countBy returns key -> count with the same key ordering. Do not mutate the input.

## Edge cases

- Empty input -> empty Map.
- keyFn receives the index as second arg in groupBy.
- Numeric key 1 and string key '1' are distinct Map keys.

## Examples

- groupBy(['apple','avocado','banana'], s => s[0]) -> Map{'a'=>['apple','avocado'],'b'=>['banana']}
- groupBy([1,2,3,4,5,6], n => n % 3) -> Map{1=>[1,4],2=>[2,5],0=>[3,6]}
- countBy(['x','y','x'], s => s) -> Map{'x'=>2,'y'=>1}
- groupBy([], String) -> Map{}
