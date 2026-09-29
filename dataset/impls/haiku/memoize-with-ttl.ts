export function memoizeTtl<A extends unknown[], R>(
  fn: (...args: A) => R,
  ttlMs: number,
  opts?: {
    key?: (...args: A) => string;
    now?: () => number;
    maxEntries?: number;
  },
): ((...args: A) => R) & { clear(): void } {
  const keyFn = opts?.key ?? ((...args: A) => JSON.stringify(args));
  const now = opts?.now ?? (() => Date.now());
  const maxEntries = opts?.maxEntries;

  interface CacheEntry {
    value: R;
    timestamp: number;
    insertOrder: number;
  }

  const cache = new Map<string, CacheEntry>();
  let insertCounter = 0;

  function memoized(...args: A): R {
    const key = keyFn(...args);

    if (cache.has(key)) {
      const entry = cache.get(key);
      if (entry && now() - entry.timestamp < ttlMs) {
        return entry.value;
      }
      cache.delete(key);
    }

    const result = fn(...args);

    if (ttlMs > 0) {
      const entry: CacheEntry = {
        value: result,
        timestamp: now(),
        insertOrder: insertCounter++,
      };

      cache.set(key, entry);

      if (maxEntries && cache.size > maxEntries) {
        let oldestKey = key;
        let oldestOrder = entry.insertOrder;
        for (const [k, v] of cache.entries()) {
          if (v.insertOrder < oldestOrder) {
            oldestKey = k;
            oldestOrder = v.insertOrder;
          }
        }
        cache.delete(oldestKey);
      }
    }

    return result;
  }

  memoized.clear = () => {
    cache.clear();
  };

  return memoized;
}
