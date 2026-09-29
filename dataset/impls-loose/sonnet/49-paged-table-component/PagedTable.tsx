import { type ReactNode, useEffect, useState } from "react";

export interface Column<T> {
  key: string;
  header: ReactNode;
  render: (row: T) => ReactNode;
}

export interface PagedTableProps<T> {
  rows: readonly T[];
  columns: readonly Column<T>[];
  pageSize?: number;
  getRowKey?: (row: T, index: number) => string | number;
  emptyMessage?: ReactNode;
}

export function PagedTable<T>({
  rows,
  columns,
  pageSize = 10,
  getRowKey = (_row, index) => index,
  emptyMessage = "No data",
}: PagedTableProps<T>) {
  const [page, setPage] = useState(0);
  const pageCount = Math.max(1, Math.ceil(rows.length / pageSize));

  useEffect(() => {
    setPage((p) => Math.min(p, pageCount - 1));
  }, [pageCount]);

  const current = Math.min(page, pageCount - 1);
  const start = current * pageSize;
  const visible = rows.slice(start, start + pageSize);

  return (
    <div>
      <table>
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column.key} scope="col">
                {column.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {visible.length === 0 ? (
            <tr>
              <td colSpan={columns.length}>{emptyMessage}</td>
            </tr>
          ) : (
            visible.map((row, i) => (
              <tr key={getRowKey(row, start + i)}>
                {columns.map((column) => (
                  <td key={column.key}>{column.render(row)}</td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
      <nav aria-label="Pagination">
        <button type="button" onClick={() => setPage(current - 1)} disabled={current === 0}>
          Previous
        </button>
        <span aria-live="polite">
          {" "}
          Page {current + 1} of {pageCount}{" "}
        </span>
        <button
          type="button"
          onClick={() => setPage(current + 1)}
          disabled={current >= pageCount - 1}
        >
          Next
        </button>
      </nav>
    </div>
  );
}
