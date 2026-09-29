import { useState } from "react";

export interface Column<T> {
  key: keyof T & string;
  header: string;
  render?: (row: T) => string | number;
}

export interface PagedTableProps<T> {
  rows: readonly T[];
  columns: readonly Column<T>[];
  pageSize?: number;
  getRowId: (row: T) => string | number;
}

export function PagedTable<T>(props: PagedTableProps<T>) {
  const { rows, columns, pageSize: propPageSize, getRowId } = props;

  const pageSize = propPageSize && propPageSize > 0 ? propPageSize : 10;
  const totalPages = Math.max(1, Math.ceil(rows.length / pageSize));

  const [currentPage, setCurrentPage] = useState(0);

  // Clamp page if out of range
  const clampedPage = Math.min(currentPage, totalPages - 1);
  const displayPage = clampedPage + 1;

  const startIdx = clampedPage * pageSize;
  const endIdx = startIdx + pageSize;
  const currentRows = rows.slice(startIdx, endIdx);

  return (
    <>
      <table>
        <thead>
          <tr>
            {columns.map((col) => (
              <th key={String(col.key)}>{col.header}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {currentRows.length === 0 ? (
            <tr>
              <td colSpan={columns.length}>No data</td>
            </tr>
          ) : (
            currentRows.map((row) => (
              <tr key={getRowId(row)}>
                {columns.map((col) => (
                  <td key={String(col.key)}>
                    {col.render ? col.render(row) : String(row[col.key] ?? "")}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
      <nav>
        <button
          type="button"
          onClick={() => setCurrentPage(Math.max(0, clampedPage - 1))}
          disabled={clampedPage === 0}
        >
          Previous
        </button>
        <span>
          Page {displayPage} of {totalPages}
        </span>
        <button
          type="button"
          onClick={() => setCurrentPage(Math.min(totalPages - 1, clampedPage + 1))}
          disabled={clampedPage >= totalPages - 1}
        >
          Next
        </button>
      </nav>
    </>
  );
}
