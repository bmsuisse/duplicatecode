export type PageItem = number | "ellipsis";

export function getPageItems(
  current: number,
  totalPages: number,
  opts: { siblings?: number; boundaries?: number } = {},
): PageItem[] {
  if (totalPages < 1) {
    return [];
  }
  const { siblings = 1, boundaries = 1 } = opts;
  if (totalPages <= 2 * boundaries + 2 * siblings + 3) {
    return Array.from({ length: totalPages }, (_, i) => i + 1);
  }
  const page = Math.min(Math.max(current, 1), totalPages);
  const wanted = new Set<number>();
  const add = (from: number, to: number) => {
    for (let n = Math.max(from, 1); n <= Math.min(to, totalPages); n++) {
      wanted.add(n);
    }
  };
  add(1, boundaries);
  add(totalPages - boundaries + 1, totalPages);
  add(page - siblings, page + siblings);

  const items: PageItem[] = [];
  let previous = 0;
  for (const n of [...wanted].sort((a, b) => a - b)) {
    if (n - previous === 2) {
      items.push(previous + 1);
    } else if (n - previous > 2) {
      items.push("ellipsis");
    }
    items.push(n);
    previous = n;
  }
  return items;
}
