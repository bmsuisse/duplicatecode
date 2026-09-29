export type SortDirection = "asc" | "desc";

export interface SortSpec<T> {
  key: keyof T;
  direction?: SortDirection;
}

function compareValues(a: unknown, b: unknown): number {
  if (a === b) return 0;
  if (a === null || a === undefined) return 1; // empty values always sort last
  if (b === null || b === undefined) return -1;
  if (typeof a === "number" && typeof b === "number") return a - b;
  if (a instanceof Date && b instanceof Date) return a.getTime() - b.getTime();
  if (typeof a === "string" && typeof b === "string") {
    return a.localeCompare(b, undefined, { numeric: true, sensitivity: "base" });
  }
  return String(a) < String(b) ? -1 : String(a) > String(b) ? 1 : 0;
}

/** Create a comparator that sorts by the first spec and breaks ties with the following ones. */
export function createComparator<T>(specs: ReadonlyArray<SortSpec<T>>): (a: T, b: T) => number {
  return (a, b) => {
    for (const { key, direction = "asc" } of specs) {
      const left = a[key];
      const right = b[key];
      const leftEmpty = left === null || left === undefined;
      const rightEmpty = right === null || right === undefined;
      if (leftEmpty || rightEmpty) {
        if (leftEmpty && rightEmpty) continue;
        return leftEmpty ? 1 : -1;
      }
      const result = compareValues(left, right);
      if (result !== 0) return direction === "desc" ? -result : result;
    }
    return 0;
  };
}

/** Return a sorted copy; the input array is not modified. The sort is stable. */
export function sortByKeys<T>(items: readonly T[], specs: ReadonlyArray<SortSpec<T>>): T[] {
  return [...items].sort(createComparator(specs));
}
