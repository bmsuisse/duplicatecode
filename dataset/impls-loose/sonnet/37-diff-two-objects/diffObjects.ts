export interface ObjectDiff<T extends Record<string, unknown>> {
  added: Partial<T>;
  removed: Partial<T>;
  changed: Record<string, { from: unknown; to: unknown }>;
}

function isEqual(a: unknown, b: unknown): boolean {
  if (Object.is(a, b)) return true;
  if (a instanceof Date && b instanceof Date) return a.getTime() === b.getTime();
  if (typeof a !== "object" || typeof b !== "object" || a === null || b === null) return false;
  return JSON.stringify(a) === JSON.stringify(b);
}

/** Compare two flat objects, optionally ignoring some keys (e.g. timestamps). */
export function diffObjects<T extends Record<string, unknown>>(
  before: T,
  after: T,
  ignoreKeys: readonly string[] = [],
): ObjectDiff<T> {
  const ignored = new Set(ignoreKeys);
  const diff: ObjectDiff<T> = { added: {}, removed: {}, changed: {} };
  const has = (o: object, k: string): boolean => Object.hasOwn(o, k);

  for (const key of Object.keys(after)) {
    if (ignored.has(key) || has(before, key)) continue;
    (diff.added as Record<string, unknown>)[key] = after[key];
  }
  for (const key of Object.keys(before)) {
    if (ignored.has(key)) continue;
    if (!has(after, key)) {
      (diff.removed as Record<string, unknown>)[key] = before[key];
    } else if (!isEqual(before[key], after[key])) {
      diff.changed[key] = { from: before[key], to: after[key] };
    }
  }
  return diff;
}
