export interface MemoizeOptions<A extends unknown[]> {
  /** Time to live of a cached result in milliseconds. */
  ttlMs: number;
  /** Build the cache key from the arguments (default: JSON of the arguments). */
  keyOf?: (...args: A) => string;
  /** Injectable clock, mainly for tests. */
  now?: () => number;
}

export interface Memoized<A extends unknown[], R> {
  (...args: A): R;
  clear(): void;
}

/** Cache results of `fn` per argument list; entries expire `ttlMs` after being computed. */
export function memoizeWithTtl<A extends unknown[], R>(
  fn: (...args: A) => R,
  options: MemoizeOptions<A>,
): Memoized<A, R> {
  const { ttlMs, keyOf = (...args: A) => JSON.stringify(args), now = Date.now } = options;
  const cache = new Map<string, { value: R; expiresAt: number }>();

  const memoized = (...args: A): R => {
    const key = keyOf(...args);
    const time = now();
    const hit = cache.get(key);
    if (hit && hit.expiresAt > time) return hit.value;
    const value = fn(...args);
    cache.set(key, { value, expiresAt: time + ttlMs });
    if (value instanceof Promise) {
      // Do not keep rejected promises around.
      value.catch(() => {
        if (cache.get(key)?.value === value) cache.delete(key);
      });
    }
    // Opportunistic cleanup so the cache does not grow without bound.
    for (const [k, entry] of cache) {
      if (entry.expiresAt <= time) cache.delete(k);
    }
    return value;
  };

  memoized.clear = () => cache.clear();
  return memoized;
}
