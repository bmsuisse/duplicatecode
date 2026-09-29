type Plain = Record<string, unknown>;
type ArrayMode = "replace" | "concat" | "unique";

const BLOCKED_KEYS = new Set(["__proto__", "constructor", "prototype"]);

function isPlain(value: unknown): value is Plain {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const proto = Object.getPrototypeOf(value);
  return proto === Object.prototype || proto === null;
}

function clone(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(clone);
  }
  if (isPlain(value)) {
    return mergeInto({}, value, "replace");
  }
  return value;
}

function mergeInto(target: Plain, source: Plain, arrays: ArrayMode): Plain {
  for (const key of Object.keys(source)) {
    const incoming = source[key];
    if (BLOCKED_KEYS.has(key) || incoming === undefined) {
      continue;
    }
    const existing = Object.hasOwn(target, key) ? target[key] : undefined;
    if (isPlain(existing) && isPlain(incoming)) {
      target[key] = mergeInto(existing, incoming, arrays);
    } else if (Array.isArray(existing) && Array.isArray(incoming) && arrays !== "replace") {
      const combined = [...existing, ...incoming.map(clone)];
      target[key] = arrays === "unique" ? [...new Set(combined)] : combined;
    } else {
      target[key] = clone(incoming);
    }
  }
  return target;
}

export function deepMerge<T extends Record<string, unknown>, U extends Record<string, unknown>>(
  base: T,
  override: U,
  opts: { arrays?: "replace" | "concat" | "unique" } = {},
): T & U {
  const arrays = opts.arrays ?? "replace";
  const result = mergeInto({}, base, "replace");
  return mergeInto(result, override, arrays) as T & U;
}
