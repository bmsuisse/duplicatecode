type Handler<P> = (payload: P) => void;

/**
 * Minimal typed event emitter. `Events` maps each event name to its payload type:
 * `new TypedEmitter<{ login: { user: string }; logout: undefined }>()`.
 */
export class TypedEmitter<Events extends { [K in keyof Events]: unknown }> {
  private readonly handlers = new Map<keyof Events, Set<Handler<never>>>();

  /** Subscribe to an event. Returns a function that removes the subscription. */
  on<K extends keyof Events>(event: K, handler: Handler<Events[K]>): () => void {
    let set = this.handlers.get(event);
    if (!set) {
      set = new Set();
      this.handlers.set(event, set);
    }
    set.add(handler as Handler<never>);
    return () => this.off(event, handler);
  }

  /** Subscribe for a single delivery. */
  once<K extends keyof Events>(event: K, handler: Handler<Events[K]>): () => void {
    const off = this.on(event, (payload) => {
      off();
      handler(payload);
    });
    return off;
  }

  off<K extends keyof Events>(event: K, handler: Handler<Events[K]>): void {
    const set = this.handlers.get(event);
    if (!set) return;
    set.delete(handler as Handler<never>);
    if (set.size === 0) this.handlers.delete(event);
  }

  emit<K extends keyof Events>(
    event: K,
    ...args: Events[K] extends undefined ? [payload?: Events[K]] : [payload: Events[K]]
  ): void {
    const set = this.handlers.get(event);
    if (!set) return;
    // Copy so handlers may unsubscribe while being notified.
    for (const handler of [...set]) (handler as Handler<Events[K] | undefined>)(args[0]);
  }

  listenerCount<K extends keyof Events>(event: K): number {
    return this.handlers.get(event)?.size ?? 0;
  }

  removeAllListeners(event?: keyof Events): void {
    if (event === undefined) this.handlers.clear();
    else this.handlers.delete(event);
  }
}
