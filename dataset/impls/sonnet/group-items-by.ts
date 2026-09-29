export function groupBy<T, K extends string | number>(
  items: readonly T[],
  keyFn: (item: T, index: number) => K,
): Map<K, T[]> {
  const groups = new Map<K, T[]>();
  items.forEach((item, index) => {
    const key = keyFn(item, index);
    const bucket = groups.get(key);
    if (bucket) {
      bucket.push(item);
    } else {
      groups.set(key, [item]);
    }
  });
  return groups;
}

export function countBy<T, K extends string | number>(
  items: readonly T[],
  keyFn: (item: T) => K,
): Map<K, number> {
  const counts = new Map<K, number>();
  for (const item of items) {
    const key = keyFn(item);
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  return counts;
}
