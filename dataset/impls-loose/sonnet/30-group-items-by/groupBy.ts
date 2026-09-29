/** Group items into arrays keyed by `keyFn(item)`, preserving order. */
export function groupBy<T, K>(items: Iterable<T>, keyFn: (item: T) => K): Map<K, T[]> {
  const groups = new Map<K, T[]>();
  for (const item of items) {
    const key = keyFn(item);
    const bucket = groups.get(key);
    if (bucket) bucket.push(item);
    else groups.set(key, [item]);
  }
  return groups;
}

/** Count how many items fall under each derived key. */
export function countBy<T, K>(items: Iterable<T>, keyFn: (item: T) => K): Map<K, number> {
  const counts = new Map<K, number>();
  for (const item of items) {
    const key = keyFn(item);
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  return counts;
}
