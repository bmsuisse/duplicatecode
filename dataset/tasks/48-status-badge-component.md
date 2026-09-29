# StatusBadge component

- **Language:** typescript
- **Target file:** `status-badge-component.tsx`
- **Constraints:** No npm packages other than `react` (types for react are NOT installed, so avoid importing type names from react; the file must lint cleanly with biome and use only explicit local types). Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```tsx
export type Status = 'active' | 'pending' | 'inactive' | 'error'
export interface StatusBadgeProps { status: Status; label?: string; size?: 'sm' | 'md' }
export function StatusBadge(props: StatusBadgeProps)
```

## Behavior

Function component in `status-badge-component.tsx`. Use the automatic JSX runtime (no imports needed); do not import React types and do not use the React/JSX namespace types. Renders <span> with className `badge badge--${status} badge--${size}` (size default 'md'), attribute role='status', data-status={status}, and text = label if provided and non-empty else a default: active 'Active', pending 'Pending', inactive 'Inactive', error 'Error'. Unknown status values at runtime (cast) render the raw string with class 'badge badge--unknown badge--md'.

## Edge cases

- Empty-string label falls back to default text.
- No inline styles.

## Examples

- <StatusBadge status='active' /> -> <span class='badge badge--active badge--md' role='status' data-status='active'>Active</span>
- <StatusBadge status='error' label='Failed' size='sm' /> -> class 'badge badge--error badge--sm', text 'Failed'
- status 'weird' (cast) -> class 'badge badge--unknown badge--md', text 'weird'
- label='' -> default text
