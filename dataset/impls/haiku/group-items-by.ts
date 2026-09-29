export function groupBy<T, K extends string | number>(
  items: readonly T[],
  keyFn: (item: T, index: number) => K,
): Map<K, T[]> {
  const result = new Map<K, T[]>();

  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    const key = keyFn(item, i);

    if (!result.has(key)) {
      result.set(key, []);
    }
    const arr = result.get(key);
    if (arr) arr.push(item);
  }

  return result;
}

export function countBy<T, K extends string | number>(
  items: readonly T[],
  keyFn: (item: T) => K,
): Map<K, number> {
  const result = new Map<K, number>();

  for (const item of items) {
    const key = keyFn(item);
    result.set(key, (result.get(key) ?? 0) + 1);
  }

  return result;
}
