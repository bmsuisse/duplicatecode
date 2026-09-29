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

export function PagedTable<T>({ rows, columns, pageSize = 10, getRowId }: PagedTableProps<T>) {
  const [page, setPage] = useState(0);
  const size = pageSize < 1 ? 10 : pageSize;
  const totalPages = Math.max(1, Math.ceil(rows.length / size));
  const current = Math.min(page, totalPages - 1);
  const visible = rows.slice(current * size, (current + 1) * size);

  return (
    <div>
      <table>
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column.key}>{column.header}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {visible.length === 0 ? (
            <tr>
              <td colSpan={columns.length}>No data</td>
            </tr>
          ) : (
            visible.map((row) => (
              <tr key={getRowId(row)}>
                {columns.map((column) => (
                  <td key={column.key}>
                    {column.render ? column.render(row) : String(row[column.key] ?? "")}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
      <nav>
        <button type="button" disabled={current === 0} onClick={() => setPage(current - 1)}>
          Previous
        </button>
        <span>
          Page {current + 1} of {totalPages}
        </span>
        <button
          type="button"
          disabled={current >= totalPages - 1}
          onClick={() => setPage(current + 1)}
        >
          Next
        </button>
      </nav>
    </div>
  );
}
