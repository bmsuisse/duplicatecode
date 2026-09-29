export function deepMerge<T extends Record<string, unknown>, U extends Record<string, unknown>>(
  base: T,
  override: U,
  opts?: { arrays?: "replace" | "concat" | "unique" },
): T & U {
  const arrayMode = opts?.arrays ?? "replace";

  function isPlainObject(obj: unknown): boolean {
    if (typeof obj !== "object" || obj === null) {
      return false;
    }
    if (Object.getPrototypeOf(obj) !== Object.prototype) {
      return false;
    }
    return true;
  }

  function mergeValues(baseVal: unknown, overrideVal: unknown): unknown {
    // Undefined in override means keep base
    if (overrideVal === undefined) {
      return baseVal;
    }

    // If both are plain objects, merge recursively
    if (isPlainObject(baseVal) && isPlainObject(overrideVal)) {
      const baseObj = baseVal as Record<string, unknown>;
      const overrideObj = overrideVal as Record<string, unknown>;
      return deepMerge(baseObj, overrideObj, opts);
    }

    // If both are arrays, apply array strategy
    if (Array.isArray(baseVal) && Array.isArray(overrideVal)) {
      if (arrayMode === "replace") {
        return [...overrideVal];
      } else if (arrayMode === "concat") {
        return [...baseVal, ...overrideVal];
      } else if (arrayMode === "unique") {
        const seen = new Set<unknown>();
        const result: unknown[] = [];
        for (const item of [...baseVal, ...overrideVal]) {
          if (!seen.has(item)) {
            seen.add(item);
            result.push(item);
          }
        }
        return result;
      }
    }

    // Override wins for all other cases
    return overrideVal;
  }

  const result: Record<string, unknown> = {};

  // Copy all keys from base
  for (const key of Object.keys(base)) {
    if (key !== "__proto__" && key !== "constructor" && key !== "prototype") {
      result[key] = base[key];
    }
  }

  // Merge in override
  for (const key of Object.keys(override)) {
    if (key !== "__proto__" && key !== "constructor" && key !== "prototype") {
      result[key] = mergeValues(result[key], override[key]);
    }
  }

  return result as T & U;
}
