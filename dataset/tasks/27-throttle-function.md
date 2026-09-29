# Throttle a function

- **Language:** typescript
- **Target file:** `throttle-function.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export function throttle<A extends unknown[]>(fn: (...args: A) => void, intervalMs: number, options?: { leading?: boolean; trailing?: boolean }): ((...args: A) => void) & { cancel(): void }
```

## Behavior

Limits `fn` to at most one call per `intervalMs`. Defaults leading=true, trailing=true. A call when no window is active runs immediately (if leading) and opens a window. Calls during a window save the latest args; when the window ends, if trailing and args were saved, run fn with them and open a new window. If no calls occurred during the window the throttle goes idle. Use setTimeout and Date.now-free logic (timer-driven).

## Edge cases

- leading=false: first call only schedules trailing execution at window end.
- cancel clears timer and pending args, returning to idle.

## Examples

- interval 100, calls at t=0,30,60 -> fn at t=0 (first args) and t=100 (third call's args)
- single call at t=0 -> fn once at t=0
- leading=false, call at t=0 -> fn at t=100
- trailing=false, calls t=0,30 -> only t=0
