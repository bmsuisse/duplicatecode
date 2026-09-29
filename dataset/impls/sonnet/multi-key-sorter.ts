export type SortSpec<T> = { key: keyof T; dir?: "asc" | "desc"; nulls?: "first" | "last" };

function compareValues(a: unknown, b: unknown): number {
  if (typeof a === "number" && typeof b === "number") {
    return a - b;
  }
  if (a instanceof Date && b instanceof Date) {
    return a.getTime() - b.getTime();
  }
  if (typeof a === "boolean" && typeof b === "boolean") {
    return Number(a) - Number(b);
  }
  return String(a).localeCompare(String(b), undefined, { sensitivity: "base", numeric: true });
}

export function sortBy<T>(items: readonly T[], specs: readonly SortSpec<T>[]): T[] {
  return [...items].sort((left, right) => {
    for (const { key, dir = "asc", nulls = "last" } of specs) {
      const a = left[key];
      const b = right[key];
      const aMissing = a === null || a === undefined;
      const bMissing = b === null || b === undefined;
      let result: number;
      if (aMissing || bMissing) {
        if (aMissing && bMissing) {
          continue;
        }
        result = (aMissing ? 1 : -1) * (nulls === "first" ? -1 : 1);
      } else {
        result = compareValues(a, b) * (dir === "desc" ? -1 : 1);
      }
      if (result !== 0) {
        return result;
      }
    }
    return 0;
  });
}
