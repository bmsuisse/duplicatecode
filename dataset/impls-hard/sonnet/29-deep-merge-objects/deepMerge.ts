type PlainObject = Record<string, unknown>;

function isPlainObject(value: unknown): value is PlainObject {
  if (typeof value !== "object" || value === null) return false;
  const proto = Object.getPrototypeOf(value);
  return proto === Object.prototype || proto === null;
}

function cloneValue(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(cloneValue);
  if (isPlainObject(value)) return mergeInto({}, value);
  return value;
}

function mergeInto(target: PlainObject, source: PlainObject): PlainObject {
  for (const [key, value] of Object.entries(source)) {
    if (key === "__proto__" || key === "constructor" || key === "prototype") continue;
    const existing = target[key];
    if (isPlainObject(existing) && isPlainObject(value)) {
      target[key] = mergeInto(existing, value);
    } else if (value !== undefined) {
      target[key] = cloneValue(value);
    }
  }
  return target;
}

/**
 * Return a new object where `overrides` win over `defaults`. Nested plain objects are
 * merged recursively, arrays and other values are replaced. `undefined` overrides are
 * ignored. Neither input is modified.
 */
export function deepMerge<A extends PlainObject, B extends PlainObject>(
  defaults: A,
  overrides: B,
): A & B {
  const result = mergeInto({}, defaults);
  return mergeInto(result, overrides) as A & B;
}
