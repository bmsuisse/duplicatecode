export type Diff = {
  added: Record<string, unknown>;
  removed: Record<string, unknown>;
  changed: Record<string, { from: unknown; to: unknown }>;
};

function isEqual(a: unknown, b: unknown): boolean {
  if (typeof a === "number" && typeof b === "number") {
    return a === b || (Number.isNaN(a) && Number.isNaN(b));
  }
  if (a instanceof Date && b instanceof Date) {
    return a.getTime() === b.getTime();
  }
  if (Array.isArray(a) && Array.isArray(b)) {
    return a.length === b.length && a.every((item, i) => isEqual(item, b[i]));
  }
  if (typeof a === "object" && typeof b === "object" && a !== null && b !== null) {
    if (Array.isArray(a) || Array.isArray(b) || a instanceof Date || b instanceof Date) {
      return false;
    }
    const left = a as Record<string, unknown>;
    const right = b as Record<string, unknown>;
    const keys = Object.keys(left);
    return (
      keys.length === Object.keys(right).length &&
      keys.every((key) => Object.hasOwn(right, key) && isEqual(left[key], right[key]))
    );
  }
  return Object.is(a, b);
}

export function diffObjects(
  before: Record<string, unknown>,
  after: Record<string, unknown>,
  opts: { ignore?: readonly string[] } = {},
): Diff {
  const ignored = new Set(opts.ignore ?? []);
  const diff: Diff = { added: {}, removed: {}, changed: {} };
  for (const key of Object.keys(before)) {
    if (ignored.has(key)) {
      continue;
    }
    if (!Object.hasOwn(after, key)) {
      diff.removed[key] = before[key];
    } else if (!isEqual(before[key], after[key])) {
      diff.changed[key] = { from: before[key], to: after[key] };
    }
  }
  for (const key of Object.keys(after)) {
    if (!ignored.has(key) && !Object.hasOwn(before, key)) {
      diff.added[key] = after[key];
    }
  }
  return diff;
}
