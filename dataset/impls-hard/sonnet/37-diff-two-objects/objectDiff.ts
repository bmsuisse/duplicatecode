export interface ObjectDiff<T extends Record<string, unknown>> {
  added: Partial<T>;
  removed: Partial<T>;
  changed: Partial<Record<keyof T, { from: unknown; to: unknown }>>;
}

function isEqual(a: unknown, b: unknown): boolean {
  if (Object.is(a, b)) return true;
  if (a instanceof Date && b instanceof Date) return a.getTime() === b.getTime();
  if (Array.isArray(a) && Array.isArray(b)) {
    return a.length === b.length && a.every((item, i) => isEqual(item, b[i]));
  }
  if (typeof a === "object" && typeof b === "object" && a !== null && b !== null) {
    const left = a as Record<string, unknown>;
    const right = b as Record<string, unknown>;
    const keys = Object.keys(left);
    return (
      keys.length === Object.keys(right).length &&
      keys.every((k) => Object.hasOwn(right, k) && isEqual(left[k], right[k]))
    );
  }
  return false;
}

/** Compare two flat objects and report added, removed and changed properties. */
export function diffObjects<T extends Record<string, unknown>>(
  before: T,
  after: T,
  ignore: readonly string[] = [],
): ObjectDiff<T> {
  const skip = new Set(ignore);
  const diff: ObjectDiff<T> = { added: {}, removed: {}, changed: {} };
  for (const key of Object.keys(after) as Array<keyof T & string>) {
    if (skip.has(key)) continue;
    if (!Object.hasOwn(before, key)) diff.added[key] = after[key];
    else if (!isEqual(before[key], after[key])) {
      diff.changed[key] = { from: before[key], to: after[key] };
    }
  }
  for (const key of Object.keys(before) as Array<keyof T & string>) {
    if (skip.has(key)) continue;
    if (!Object.hasOwn(after, key)) diff.removed[key] = before[key];
  }
  return diff;
}

export function hasChanges<T extends Record<string, unknown>>(diff: ObjectDiff<T>): boolean {
  return [diff.added, diff.removed, diff.changed].some((part) => Object.keys(part).length > 0);
}
