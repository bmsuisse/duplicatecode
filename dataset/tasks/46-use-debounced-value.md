# React hook: useDebouncedValue

- **Language:** typescript
- **Target file:** `use-debounced-value.ts`
- **Constraints:** No npm packages other than `react` (types for react are NOT installed, so avoid importing type names from react; the file must lint cleanly with biome and use only explicit local types). Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export function useDebouncedValue<T>(value: T, delayMs: number): T
```

## Behavior

Write in `use-debounced-value.ts`. Import only { useEffect, useState } from 'react'. Returns the initial value on first render; after `value` changes, the returned value updates to the latest value only once `delayMs` ms passed without further changes. Effect cleanup must clear the timeout on change and unmount. delayMs <= 0 still goes through setTimeout(0).

## Edge cases

- Rapid successive changes only produce the last value.
- Changing delayMs restarts the timer.
- Must not call setState after unmount.

## Examples

- initial 'a' -> returns 'a'
- value 'a' -> 'ab' -> 'abc' within delay -> returns 'a' until delay, then 'abc'
- unmount before the delay -> no update, no warnings
- same value re-rendered -> no extra state changes needed
