# Pagination page list with ellipses

- **Language:** typescript
- **Target file:** `pagination-page-list.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export type PageItem = number | 'ellipsis'
export function getPageItems(current: number, totalPages: number, opts?: { siblings?: number; boundaries?: number }): PageItem[]
```

## Behavior

Returns page buttons for a pager. Defaults siblings=1, boundaries=1. Always includes the first `boundaries` and last `boundaries` pages, the current page and `siblings` pages each side. Gaps of exactly one page are filled with that page number instead of an ellipsis; larger gaps become a single 'ellipsis'. current is clamped to [1,totalPages]. totalPages < 1 -> [].

## Edge cases

- Small totals return all pages.
- Result never has two consecutive ellipses.

## Examples

- (1, 5) -> [1,2,3,4,5]
- (1, 10) -> [1,2,'ellipsis',10]
- (5, 10) -> [1,'ellipsis',4,5,6,'ellipsis',10]
- (4, 10) -> [1,2,3,4,5,'ellipsis',10]
- (99, 10) -> [1,'ellipsis',9,10]
- (1, 0) -> []
