export type PageItem = number | "ellipsis";

export interface PageListOptions {
  /** Pages always shown at the start and the end. */
  edgeCount?: number;
  /** Pages shown on each side of the current page. */
  siblingCount?: number;
}

function range(from: number, to: number): number[] {
  return Array.from({ length: Math.max(0, to - from + 1) }, (_, i) => from + i);
}

/**
 * Compute which page numbers to show, e.g. [1, "ellipsis", 4, 5, 6, "ellipsis", 20].
 * A gap of exactly one page is filled with that page instead of an ellipsis.
 */
export function getPageList(
  currentPage: number,
  totalPages: number,
  { edgeCount = 1, siblingCount = 1 }: PageListOptions = {},
): PageItem[] {
  if (totalPages < 1) return [];
  const current = Math.min(Math.max(1, Math.trunc(currentPage)), totalPages);
  const shown = new Set<number>([
    ...range(1, Math.min(edgeCount, totalPages)),
    ...range(Math.max(totalPages - edgeCount + 1, 1), totalPages),
    ...range(Math.max(1, current - siblingCount), Math.min(totalPages, current + siblingCount)),
  ]);
  const pages = [...shown].sort((a, b) => a - b);
  const result: PageItem[] = [];
  let previous = 0;
  for (const page of pages) {
    const gap = page - previous;
    if (gap === 2) result.push(previous + 1);
    else if (gap > 2) result.push("ellipsis");
    result.push(page);
    previous = page;
  }
  return result;
}
