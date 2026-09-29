export function memoizeTtl<A extends unknown[], R>(
  fn: (...args: A) => R,
  ttlMs: number,
  opts: { key?: (...args: A) => string; now?: () => number; maxEntries?: number } = {},
): ((...args: A) => R) & { clear(): void } {
  const { key = (...args: A) => JSON.stringify(args), now = Date.now, maxEntries } = opts;
  const cache = new Map<string, { value: R; storedAt: number }>();

  const memoized = (...args: A): R => {
    if (ttlMs <= 0) {
      return fn(...args);
    }
    const cacheKey = key(...args);
    const hit = cache.get(cacheKey);
    if (hit && now() - hit.storedAt < ttlMs) {
      return hit.value;
    }
    const value = fn(...args);
    cache.delete(cacheKey);
    cache.set(cacheKey, { value, storedAt: now() });
    if (maxEntries !== undefined && cache.size > maxEntries) {
      for (const oldest of cache.keys()) {
        cache.delete(oldest);
        break;
      }
    }
    return value;
  };

  memoized.clear = (): void => {
    cache.clear();
  };

  return memoized;
}
