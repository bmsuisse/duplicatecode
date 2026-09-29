export interface PaginationList {
  pages: (number | string)[];
  currentPage: number;
  totalPages: number;
}

export function getPaginationPages(
  currentPage: number,
  totalPages: number,
  maxVisible: number = 7,
): PaginationList {
  const pages: (number | string)[] = [];
  let start = Math.max(1, currentPage - Math.floor(maxVisible / 2));
  const end = Math.min(totalPages, start + maxVisible - 1);

  if (end - start < maxVisible - 1) {
    start = Math.max(1, end - maxVisible + 1);
  }

  if (start > 1) {
    pages.push(1);
    if (start > 2) {
      pages.push("...");
    }
  }

  for (let i = start; i <= end; i++) {
    pages.push(i);
  }

  if (end < totalPages) {
    if (end < totalPages - 1) {
      pages.push("...");
    }
    pages.push(totalPages);
  }

  return { pages, currentPage, totalPages };
}
