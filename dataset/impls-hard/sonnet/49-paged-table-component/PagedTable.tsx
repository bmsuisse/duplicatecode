import { type ReactNode, useEffect, useState } from "react";

export interface TableColumn<T> {
  key: string;
  header: ReactNode;
  render: (row: T) => ReactNode;
  align?: "left" | "right" | "center";
}

export interface PagedTableProps<T> {
  rows: readonly T[];
  columns: ReadonlyArray<TableColumn<T>>;
  /** Stable identifier for each row. */
  rowKey: (row: T) => string | number;
  pageSize?: number;
  emptyMessage?: ReactNode;
}

/** Table with externally described columns and previous/next paging. */
export function PagedTable<T>({
  rows,
  columns,
  rowKey,
  pageSize = 10,
  emptyMessage = "No data",
}: PagedTableProps<T>) {
  const [page, setPage] = useState(1);
  const size = Math.max(1, Math.floor(pageSize));
  const totalPages = Math.max(1, Math.ceil(rows.length / size));

  // Keep the page valid when the data shrinks.
  useEffect(() => {
    setPage((current) => Math.min(current, totalPages));
  }, [totalPages]);

  const current = Math.min(page, totalPages);
  const visible = rows.slice((current - 1) * size, current * size);

  return (
    <div>
      <table>
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column.key} style={{ textAlign: column.align ?? "left" }}>
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
            visible.map((row) => (
              <tr key={rowKey(row)}>
                {columns.map((column) => (
                  <td key={column.key} style={{ textAlign: column.align ?? "left" }}>
                    {column.render(row)}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
      <nav aria-label="Pagination" style={{ display: "flex", gap: 8, alignItems: "center" }}>
        <button type="button" onClick={() => setPage(current - 1)} disabled={current <= 1}>
          Previous
        </button>
        <span aria-live="polite">
          Page {current} of {totalPages}
        </span>
        <button type="button" onClick={() => setPage(current + 1)} disabled={current >= totalPages}>
          Next
        </button>
      </nav>
    </div>
  );
}
