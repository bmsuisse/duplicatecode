export interface SortKey {
  key: string;
  ascending?: boolean;
}

export function createMultiKeySorter(sortKeys: SortKey[]) {
  return (a: Record<string, any>, b: Record<string, any>): number => {
    for (const { key, ascending = true } of sortKeys) {
      const aVal = a[key];
      const bVal = b[key];

      let cmp = 0;
      if (aVal < bVal) cmp = -1;
      else if (aVal > bVal) cmp = 1;

      if (cmp !== 0) {
        return ascending ? cmp : -cmp;
      }
    }
    return 0;
  };
}
