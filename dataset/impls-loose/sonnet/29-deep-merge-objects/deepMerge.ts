export type PlainObject = Record<string, unknown>;

function isPlainObject(value: unknown): value is PlainObject {
  if (typeof value !== "object" || value === null) return false;
  const proto = Object.getPrototypeOf(value);
  return proto === Object.prototype || proto === null;
}

function clone<T>(value: T): T {
  if (Array.isArray(value)) return value.map(clone) as T;
  if (isPlainObject(value)) return deepMerge({}, value) as T;
  return value;
}

/**
 * Recursively merge plain objects into a new object. Later sources win;
 * arrays and other values are replaced, not merged. Inputs are never mutated.
 */
export function deepMerge(...sources: PlainObject[]): PlainObject {
  const result: PlainObject = {};
  for (const source of sources) {
    for (const [key, value] of Object.entries(source)) {
      if (key === "__proto__" || key === "constructor" || key === "prototype") continue;
      const existing = result[key];
      result[key] =
        isPlainObject(existing) && isPlainObject(value) ? deepMerge(existing, value) : clone(value);
    }
  }
  return result;
}
