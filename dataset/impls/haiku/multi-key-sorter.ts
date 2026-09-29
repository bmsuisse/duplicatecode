export type SortSpec<T> = { key: keyof T; dir?: "asc" | "desc"; nulls?: "first" | "last" };

export function sortBy<T>(items: readonly T[], specs: readonly SortSpec<T>[]): T[] {
  const result = [...items];

  result.sort((a, b) => {
    for (const spec of specs) {
      const aVal = a[spec.key];
      const bVal = b[spec.key];
      const dir = spec.dir ?? "asc";
      const nulls = spec.nulls ?? "last";

      const aIsNull = aVal === null || aVal === undefined;
      const bIsNull = bVal === null || bVal === undefined;

      if (aIsNull || bIsNull) {
        if (aIsNull && bIsNull) {
          continue;
        }
        if (nulls === "first") {
          return aIsNull ? -1 : 1;
        } else {
          return aIsNull ? 1 : -1;
        }
      }

      let cmp = 0;

      if (typeof aVal === "number" && typeof bVal === "number") {
        cmp = aVal - bVal;
      } else if (typeof aVal === "string" && typeof bVal === "string") {
        cmp = aVal.localeCompare(bVal, undefined, { sensitivity: "base", numeric: true });
      } else if (aVal instanceof Date && bVal instanceof Date) {
        cmp = aVal.getTime() - bVal.getTime();
      } else if (typeof aVal === "boolean" && typeof bVal === "boolean") {
        cmp = (aVal ? 1 : 0) - (bVal ? 1 : 0);
      } else {
        // Mixed types: compare String forms
        const aStr = String(aVal);
        const bStr = String(bVal);
        cmp = aStr.localeCompare(bStr, undefined, { sensitivity: "base", numeric: true });
      }

      if (cmp !== 0) {
        return dir === "desc" ? -cmp : cmp;
      }
    }

    return 0;
  });

  return result;
}
