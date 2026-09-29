export type SortOrder = "asc" | "desc";

export interface SortKey<T> {
  key: keyof T;
  order?: SortOrder;
}

export function multiKeySort<T extends Record<string, any>>(items: T[], keys: SortKey<T>[]): T[] {
  return [...items].sort((a, b) => {
    for (const { key, order = "asc" } of keys) {
      const aVal = a[key];
      const bVal = b[key];

      if (aVal < bVal) return order === "asc" ? -1 : 1;
      if (aVal > bVal) return order === "asc" ? 1 : -1;
    }
    return 0;
  });
}
