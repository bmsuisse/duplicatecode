export function deepMerge<T extends Record<string, any>>(base: T, updates: Partial<T>): T {
  const result = { ...base };

  for (const key in updates) {
    const updateValue = updates[key];
    const baseValue = result[key];

    if (
      baseValue &&
      typeof baseValue === "object" &&
      !Array.isArray(baseValue) &&
      updateValue &&
      typeof updateValue === "object" &&
      !Array.isArray(updateValue)
    ) {
      result[key] = deepMerge(baseValue, updateValue);
    } else {
      result[key] = updateValue as any;
    }
  }

  return result;
}
