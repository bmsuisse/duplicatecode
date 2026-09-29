export interface SortKey<T> {
  key: keyof T;
  direction?: "asc" | "desc";
}

function compareValues(a: unknown, b: unknown): number {
  if (a === b) return 0;
  if (a === null || a === undefined) return 1; // nullish always last
  if (b === null || b === undefined) return -1;
  if (typeof a === "string" && typeof b === "string") {
    return a.localeCompare(b, undefined, { numeric: true, sensitivity: "base" });
  }
  if (a instanceof Date && b instanceof Date) return a.getTime() - b.getTime();
  return (a as number) < (b as number) ? -1 : (a as number) > (b as number) ? 1 : 0;
}

/** Return a new array sorted by `keys` in priority order (stable; input untouched). */
export function sortBy<T>(items: readonly T[], keys: readonly (SortKey<T> | keyof T)[]): T[] {
  const specs = keys.map((k) =>
    typeof k === "object" && k !== null
      ? (k as SortKey<T>)
      : ({ key: k as keyof T, direction: "asc" } as SortKey<T>),
  );
  return [...items].sort((x, y) => {
    for (const { key, direction = "asc" } of specs) {
      const a = x[key];
      const b = y[key];
      const aNull = a === null || a === undefined;
      const bNull = b === null || b === undefined;
      // Keep nullish last regardless of direction.
      if (aNull || bNull) {
        if (aNull && bNull) continue;
        return aNull ? 1 : -1;
      }
      const result = compareValues(a, b);
      if (result !== 0) return direction === "desc" ? -result : result;
    }
    return 0;
  });
}
