import { useState } from "react";

interface Column<T> {
  key: keyof T;
  label: string;
  render?: (value: any) => React.ReactNode;
}

interface PagedTableProps<T> {
  data: T[];
  columns: Column<T>[];
  pageSize?: number;
}

export function PagedTable<T extends Record<string, any>>({
  data,
  columns,
  pageSize = 10,
}: PagedTableProps<T>) {
  const [currentPage, setCurrentPage] = useState(1);
  const totalPages = Math.ceil(data.length / pageSize);
  const start = (currentPage - 1) * pageSize;
  const end = start + pageSize;
  const pageData = data.slice(start, end);

  return (
    <div>
      <table className="w-full">
        <thead>
          <tr>
            {columns.map((col) => (
              <th key={String(col.key)} className="border p-2">
                {col.label}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {pageData.map((row, idx) => (
            <tr key={idx}>
              {columns.map((col) => (
                <td key={String(col.key)} className="border p-2">
                  {col.render ? col.render(row[col.key]) : row[col.key]}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      <div className="mt-4 flex justify-between">
        <button
          type="button"
          onClick={() => setCurrentPage(Math.max(1, currentPage - 1))}
          disabled={currentPage === 1}
        >
          Previous
        </button>
        <span>
          Page {currentPage} of {totalPages}
        </span>
        <button
          type="button"
          onClick={() => setCurrentPage(Math.min(totalPages, currentPage + 1))}
          disabled={currentPage === totalPages}
        >
          Next
        </button>
      </div>
    </div>
  );
}
