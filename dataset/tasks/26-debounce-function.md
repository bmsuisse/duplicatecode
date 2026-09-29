# Debounce a function

- **Language:** typescript
- **Target file:** `debounce-function.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export function debounce<A extends unknown[]>(fn: (...args: A) => void, waitMs: number, options?: { leading?: boolean; trailing?: boolean }): ((...args: A) => void) & { cancel(): void; flush(): void }
```

## Behavior

Delays calling `fn` until `waitMs` ms have passed since the last invocation. Defaults: leading=false, trailing=true. Each call resets the timer and remembers the latest args. With leading=true the first call of a burst invokes `fn` immediately; with trailing also true, a trailing call happens only if there was at least one further call during the wait. `cancel` drops pending call. `flush` immediately runs a pending trailing call (if any) and clears the timer. Uses setTimeout/clearTimeout (testable with fake timers).

## Edge cases

- waitMs 0 still defers via setTimeout unless leading.
- leading=false and trailing=false never calls fn.
- flush with nothing pending is a no-op.

## Examples

- calls at t=0,50,100 with wait 100 -> fn called once at t=200 with the args of the t=100 call
- leading=true, trailing=false, calls at t=0,10 -> fn called once at t=0
- leading+trailing, single call -> fn called once (t=0)
- call then cancel() -> never called
- call then flush() -> called immediately once, not again after wait
