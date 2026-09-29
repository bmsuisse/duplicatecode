export type Diff = {
  added: Record<string, unknown>;
  removed: Record<string, unknown>;
  changed: Record<string, { from: unknown; to: unknown }>;
};

function deepEqual(a: unknown, b: unknown): boolean {
  // Handle NaN
  if (typeof a === "number" && typeof b === "number" && Number.isNaN(a) && Number.isNaN(b)) {
    return true;
  }

  // Handle Object.is-like comparison for primitives
  if (Object.is(a, b)) {
    return true;
  }

  // Handle arrays
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) {
      return false;
    }
    for (let i = 0; i < a.length; i++) {
      if (!deepEqual(a[i], b[i])) {
        return false;
      }
    }
    return true;
  }

  // Handle dates
  if (a instanceof Date && b instanceof Date) {
    return a.getTime() === b.getTime();
  }

  // Handle plain objects
  if (typeof a === "object" && typeof b === "object" && a !== null && b !== null) {
    // Check if both are plain objects
    if (
      Object.getPrototypeOf(a) === Object.prototype &&
      Object.getPrototypeOf(b) === Object.prototype
    ) {
      const aKeys = Object.keys(a);
      const bKeys = Object.keys(b);
      if (aKeys.length !== bKeys.length) {
        return false;
      }
      for (const key of aKeys) {
        if (!bKeys.includes(key)) {
          return false;
        }
        const aVal = (a as Record<string, unknown>)[key];
        const bVal = (b as Record<string, unknown>)[key];
        if (!deepEqual(aVal, bVal)) {
          return false;
        }
      }
      return true;
    }
  }

  return false;
}

export function diffObjects(
  before: Record<string, unknown>,
  after: Record<string, unknown>,
  opts?: { ignore?: readonly string[] },
): Diff {
  const ignore = new Set(opts?.ignore ?? []);

  const added: Record<string, unknown> = {};
  const removed: Record<string, unknown> = {};
  const changed: Record<string, { from: unknown; to: unknown }> = {};

  // Check for removed and changed keys
  for (const key of Object.keys(before)) {
    if (ignore.has(key)) {
      continue;
    }
    if (!(key in after)) {
      removed[key] = before[key];
    } else if (!deepEqual(before[key], after[key])) {
      changed[key] = { from: before[key], to: after[key] };
    }
  }

  // Check for added keys
  for (const key of Object.keys(after)) {
    if (ignore.has(key)) {
      continue;
    }
    if (!(key in before)) {
      added[key] = after[key];
    }
  }

  return { added, removed, changed };
}
