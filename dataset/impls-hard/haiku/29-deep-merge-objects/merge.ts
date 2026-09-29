export function deepMerge<T extends Record<string, any>>(base: T, override: Partial<T>): T {
  const result: any = { ...base };

  for (const key in override) {
    if (Object.hasOwn(override, key)) {
      const value = override[key];
      if (
        typeof value === "object" &&
        value !== null &&
        !Array.isArray(value) &&
        typeof result[key] === "object" &&
        result[key] !== null &&
        !Array.isArray(result[key])
      ) {
        result[key] = deepMerge(result[key], value);
      } else {
        result[key] = value;
      }
    }
  }

  return result as T;
}
