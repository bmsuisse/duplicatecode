export type PageItem = number | "ellipsis";

/**
 * Page numbers for a pagination bar, e.g. [1, "ellipsis", 4, 5, 6, "ellipsis", 20].
 * Always shows first/last page and `siblings` pages around the current one.
 */
export function getPageList(current: number, totalPages: number, siblings = 1): PageItem[] {
  if (totalPages < 1) return [];
  const page = Math.min(Math.max(Math.trunc(current), 1), totalPages);
  const slots = siblings * 2 + 5; // first, last, current, two ellipsis slots
  if (totalPages <= slots) {
    return Array.from({ length: totalPages }, (_, i) => i + 1);
  }

  const left = Math.max(page - siblings, 2);
  const right = Math.min(page + siblings, totalPages - 1);
  const items: PageItem[] = [1];

  if (left > 3) items.push("ellipsis");
  else for (let p = 2; p < left; p++) items.push(p);

  for (let p = left; p <= right; p++) items.push(p);

  if (right < totalPages - 2) items.push("ellipsis");
  else for (let p = right + 1; p < totalPages; p++) items.push(p);

  items.push(totalPages);
  return items;
}
