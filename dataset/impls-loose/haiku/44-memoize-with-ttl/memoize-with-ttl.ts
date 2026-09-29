export function memoizeWithTTL<T extends (...args: any[]) => any>(fn: T, ttl: number): T {
  const cache = new Map<string, { value: any; expires: number }>();

  return ((...args: any[]) => {
    const key = JSON.stringify(args);
    const cached = cache.get(key);
    const now = Date.now();

    if (cached && cached.expires > now) {
      return cached.value;
    }

    const result = fn(...args);
    cache.set(key, { value: result, expires: now + ttl });
    return result;
  }) as T;
}
