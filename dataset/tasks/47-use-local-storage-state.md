# React hook: useLocalStorageState

- **Language:** typescript
- **Target file:** `use-local-storage-state.ts`
- **Constraints:** No npm packages other than `react` (types for react are NOT installed, so avoid importing type names from react; the file must lint cleanly with biome and use only explicit local types). Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export function useLocalStorageState<T>(key: string, initial: T): [T, (next: T | ((prev: T) => T)) => void, () => void]
```

## Behavior

File `use-local-storage-state.ts`; import only { useCallback, useState } from 'react'. Returns [value, setValue, remove]. Initial state: read window.localStorage[key] and JSON.parse it; if missing, invalid JSON, unavailable (SSR: `typeof window === 'undefined'`, or access throws) use `initial`. setValue accepts a value or updater fn, updates state and writes JSON.stringify to localStorage (write errors swallowed). remove deletes the key and resets state to `initial`. Every storage access wrapped in try/catch.

## Edge cases

- Stored 'null' JSON parses to null (valid value).
- Corrupted JSON falls back to initial without throwing.

## Examples

- empty storage, initial 5 -> value 5
- storage 'theme' = '"dark"' -> value 'dark'
- setValue(v => v + 1) from 5 -> state 6 and storage '6'
- remove() -> storage key gone, value equals initial
- storage value '{bad' -> initial
