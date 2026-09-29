# Generic LRU cache

- **Language:** typescript
- **Target file:** `lru-cache-map.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export class LruCache<K, V> {
  constructor(capacity: number, onEvict?: (key: K, value: V) => void)
  get(key: K): V | undefined
  set(key: K, value: V): this
  has(key: K): boolean
  delete(key: K): boolean
  get size(): number
  keys(): K[] // least recently used first
}
```

## Behavior

Least-recently-used cache backed by a Map. `get` hit and `set` mark the key as most recent. `has`, `keys` do not affect recency. When a `set` of a new key makes size exceed capacity, evict the least recently used entry and call `onEvict(key, value)`. Overwriting an existing key does not call onEvict. `delete` does not call onEvict.

## Edge cases

- capacity < 1 or not integer -> throws RangeError.
- undefined values can be stored; use `has` to distinguish.

## Examples

- cap 2: set a, set b, set c -> keys() ['b','c'], onEvict('a', ...)
- cap 2: set a, set b, get a, set c -> keys() ['a','c']
- set a=1, set a=2 -> size 1, get a 2
- new LruCache(0) -> throws RangeError
