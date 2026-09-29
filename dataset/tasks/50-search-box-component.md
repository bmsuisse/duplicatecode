# Debounced SearchBox component

- **Language:** typescript
- **Target file:** `search-box-component.tsx`
- **Constraints:** No npm packages other than `react` (types for react are NOT installed, so avoid importing type names from react; the file must lint cleanly with biome and use only explicit local types). Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```tsx
export interface SearchBoxProps { onSearch: (query: string) => void; delayMs?: number; placeholder?: string; initialValue?: string; minLength?: number }
export function SearchBox(props: SearchBoxProps)
```

## Behavior

File `search-box-component.tsx`; import { useEffect, useRef, useState } from 'react' only. Renders <input type='search' aria-label='Search'> controlled by local state (initialValue default ''), and a 'Clear' <button> shown only when text is non-empty. Typing calls onSearch(trimmedValue) after `delayMs` (default 300) of inactivity. If trimmed length < minLength (default 0) and non-empty, onSearch is not called (but an empty value still calls onSearch('')). onSearch is not called on initial mount. Clear button empties the input and calls onSearch('') immediately (cancelling pending timer). Cleanup timers on unmount. Use a ref to hold the latest onSearch.

## Edge cases

- Whitespace-only input trims to '' and calls onSearch('').
- Same trimmed value as last emitted is not emitted again.

## Examples

- type 'a','ab','abc' quickly, delay 300 -> onSearch('abc') once
- minLength 3, type 'ab' -> no call; type 'abc' -> onSearch('abc')
- click Clear with text 'x' -> input empty, onSearch('') immediately
- initial mount -> no call
