# Typed event emitter

- **Language:** typescript
- **Target file:** `typed-event-emitter.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export class Emitter<Events extends Record<string, unknown>> {
  on<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): () => void
  once<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): () => void
  off<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): void
  emit<K extends keyof Events>(event: K, payload: Events[K]): number
  listenerCount<K extends keyof Events>(event: K): number
}
```

## Behavior

`on` registers a handler and returns an unsubscribe function. Handlers run in registration order. `once` handlers are removed before being invoked the first time. `emit` returns the number of handlers invoked. Handlers added during emit are not called for that emit; handlers removed during emit but not yet called are skipped. Registering the same handler function twice registers it twice; `off` removes one registration (the earliest).

## Edge cases

- Errors thrown by a handler propagate immediately and stop the emit.
- emit with no listeners returns 0.

## Examples

- on('a', h); emit('a', 1) -> h(1), returns 1
- once('a', h); emit twice -> h called once; second emit returns 0
- unsubscribe = on('a', h); unsubscribe(); emit -> 0
- on('a', h) twice; off('a', h) -> listenerCount 1
