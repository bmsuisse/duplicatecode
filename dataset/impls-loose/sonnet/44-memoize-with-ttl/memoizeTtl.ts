export interface MemoizeOptions<Args extends unknown[]> {
  ttlMs: number;
  /** Derive the cache key from args (default: JSON.stringify). */
  keyFn?: (...args: Args) => string;
  now?: () => number;
}

export interface Memoized<Args extends unknown[], R> {
  (...args: Args): R;
  clear(): void;
  invalidate(...args: Args): boolean;
}

/** Cache results of `fn` per arguments; each entry expires `ttlMs` after being computed. */
export function memoizeWithTtl<Args extends unknown[], R>(
  fn: (...args: Args) => R,
  { ttlMs, keyFn = (...args) => JSON.stringify(args), now = Date.now }: MemoizeOptions<Args>,
): Memoized<Args, R> {
  const cache = new Map<string, { value: R; expiresAt: number }>();

  const memoized = (...args: Args): R => {
    const key = keyFn(...args);
    const time = now();
    const hit = cache.get(key);
    if (hit && hit.expiresAt > time) return hit.value;

    const value = fn(...args);
    cache.set(key, { value, expiresAt: time + ttlMs });
    // Do not keep failed async results around.
    if (value instanceof Promise) {
      value.catch(() => {
        if (cache.get(key)?.value === value) cache.delete(key);
      });
    }
    return value;
  };
  memoized.clear = (): void => cache.clear();
  memoized.invalidate = (...args: Args): boolean => cache.delete(keyFn(...args));
  return memoized;
}
