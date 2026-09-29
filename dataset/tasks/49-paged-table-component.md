# PagedTable component

- **Language:** typescript
- **Target file:** `paged-table-component.tsx`
- **Constraints:** No npm packages other than `react` (types for react are NOT installed, so avoid importing type names from react; the file must lint cleanly with biome and use only explicit local types). Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```tsx
export interface Column<T> { key: keyof T & string; header: string; render?: (row: T) => string | number }
export interface PagedTableProps<T> { rows: readonly T[]; columns: readonly Column<T>[]; pageSize?: number; getRowId: (row: T) => string | number }
export function PagedTable<T>(props: PagedTableProps<T>)
```

## Behavior

File `paged-table-component.tsx`; import { useState } from 'react' only. Renders a <table> with <thead> (one <th> per column) and <tbody> showing the current page (pageSize default 10; page state starts at 0). Cell text = render(row) if provided else String(row[key] ?? ''). Below the table a <nav> with a 'Previous' <button> (disabled on first page), text `Page {n} of {total}` in a <span> (n 1-based; total >= 1 even with no rows) and a 'Next' <button> (disabled on last page). Rows keyed by getRowId. If rows shrink so the page is out of range, clamp during render. Empty rows -> one <tr><td colSpan={columns.length}>No data</td></tr>. Do not depend on @types/react: annotate handlers with inline types.

## Edge cases

- pageSize < 1 treated as 10.
- Clicking Next/Previous changes page by 1 within bounds.

## Examples

- 25 rows, pageSize 10 -> shows rows 1-10, 'Page 1 of 3', Previous disabled
- after two Next clicks -> rows 21-25, 'Page 3 of 3', Next disabled
- 0 rows -> 'No data', 'Page 1 of 1'
- column with render -> cell shows rendered string
