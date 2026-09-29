# Memoize with TTL

- **Language:** typescript
- **Target file:** `memoize-with-ttl.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export function memoizeTtl<A extends unknown[], R>(fn: (...args: A) => R, ttlMs: number, opts?: { key?: (...args: A) => string; now?: () => number; maxEntries?: number }): ((...args: A) => R) & { clear(): void }
```

## Behavior

Caches results per key (default key = JSON.stringify(args)). An entry is valid while now() - storedAt < ttlMs. Expired entries are recomputed. Default now = Date.now. If maxEntries is given and exceeded after insert, drop the oldest inserted entry. If fn throws, nothing is cached. Promise results are cached as returned (no special handling). `clear` removes all entries.

## Edge cases

- Falsy return values (0, undefined, '') are cached correctly.
- ttlMs <= 0 means never cache.

## Examples

- ttl 1000; call f(1) at t=0 and t=500 -> underlying fn called once; at t=1000 called again
- f(1), f(2) different keys -> 2 calls
- fn throws on first call, succeeds second -> second call recomputes
- maxEntries 2: f(1), f(2), f(3), f(1) -> fn called 4 times
