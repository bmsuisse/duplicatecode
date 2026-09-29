# Retry an async function with backoff

- **Language:** typescript
- **Target file:** `retry-async-backoff.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export async function retry<T>(fn: (attempt: number) => Promise<T>, opts?: { retries?: number; baseDelayMs?: number; factor?: number; maxDelayMs?: number; shouldRetry?: (err: unknown, attempt: number) => boolean; sleep?: (ms: number) => Promise<void> }): Promise<T>
```

## Behavior

Calls `fn(attempt)` with attempt starting at 1. On rejection, if attempts made <= retries (retries = number of extra tries, default 3) and shouldRetry(err, attempt) (default always true), waits min(maxDelayMs, baseDelayMs * factor^(attempt-1)) using the injected `sleep` (default setTimeout based), then retries. Defaults baseDelayMs=100, factor=2, maxDelayMs=10000. Otherwise rethrows the last error. No jitter.

## Edge cases

- retries: 0 -> a single call.
- shouldRetry returning false rethrows immediately, no sleep.
- No sleep after the last failure.

## Examples

- fails twice then resolves 'ok', retries 3 -> 'ok'; sleeps [100, 200]
- always rejects, retries 2 -> rejects with the last error after 3 calls; sleeps [100, 200]
- baseDelayMs 1000, factor 10, maxDelayMs 5000, 3 failures -> sleeps [1000, 5000, 5000]
- shouldRetry false on first error -> 1 call
