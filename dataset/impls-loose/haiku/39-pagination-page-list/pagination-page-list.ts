export function paginationPageList(
  currentPage: number,
  totalPages: number,
  maxButtons: number = 5,
): (number | string)[] {
  const pages: (number | string)[] = [];

  if (totalPages <= maxButtons) {
    // Show all pages
    for (let i = 1; i <= totalPages; i++) {
      pages.push(i);
    }
  } else {
    // Show first page
    pages.push(1);

    // Calculate range around current page
    const halfRange = Math.floor(maxButtons / 2);
    const start = Math.max(2, currentPage - halfRange);
    const end = Math.min(totalPages - 1, currentPage + halfRange);

    if (start > 2) {
      pages.push("...");
    }

    for (let i = start; i <= end; i++) {
      pages.push(i);
    }

    if (end < totalPages - 1) {
      pages.push("...");
    }

    // Show last page
    pages.push(totalPages);
  }

  return pages;
}
