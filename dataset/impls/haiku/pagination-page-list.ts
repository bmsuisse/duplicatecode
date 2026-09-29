export type PageItem = number | "ellipsis";

export function getPageItems(
  current: number,
  totalPages: number,
  opts?: { siblings?: number; boundaries?: number },
): PageItem[] {
  const siblings = opts?.siblings ?? 1;
  const boundaries = opts?.boundaries ?? 1;

  if (totalPages < 1) {
    return [];
  }

  // Clamp current
  current = Math.max(1, Math.min(current, totalPages));

  const pages = new Set<number>();

  // Add boundary pages
  for (let i = 1; i <= Math.min(boundaries, totalPages); i++) {
    pages.add(i);
  }
  for (let i = Math.max(boundaries + 1, totalPages - boundaries + 1); i <= totalPages; i++) {
    pages.add(i);
  }

  // Add current and siblings
  for (
    let i = Math.max(1, current - siblings);
    i <= Math.min(totalPages, current + siblings);
    i++
  ) {
    pages.add(i);
  }

  // Convert to sorted array
  const sortedPages = Array.from(pages).sort((a, b) => a - b);

  // Build result with ellipses
  const result: PageItem[] = [];
  for (let i = 0; i < sortedPages.length; i++) {
    if (i > 0) {
      const gap = sortedPages[i] - sortedPages[i - 1];
      if (gap === 2) {
        // Single missing page, add it
        result.push(sortedPages[i - 1] + 1);
      } else if (gap > 2) {
        // Gap of more than 1, add ellipsis
        result.push("ellipsis");
      }
    }
    result.push(sortedPages[i]);
  }

  return result;
}
