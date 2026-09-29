export function memoizeWithTTL<T extends (...args: any[]) => any>(fn: T, ttl: number): T {
  const cache = new Map<string, { value: any; expiry: number }>();

  return ((...args: Parameters<T>) => {
    const key = JSON.stringify(args);
    const now = Date.now();

    const entry = cache.get(key);
    if (entry !== undefined) {
      if (entry.expiry > now) {
        return entry.value;
      }
      cache.delete(key);
    }

    const value = fn(...args);
    cache.set(key, { value, expiry: now + ttl });
    return value;
  }) as T;
}
