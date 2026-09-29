export class TypedEmitter<Events extends { [K in keyof Events]: unknown }> {
  private readonly listeners = new Map<keyof Events, Set<(payload: never) => void>>();

  /** Subscribe to an event. Returns an unsubscribe function. */
  on<K extends keyof Events>(event: K, listener: (payload: Events[K]) => void): () => void {
    let set = this.listeners.get(event);
    if (!set) {
      set = new Set();
      this.listeners.set(event, set);
    }
    set.add(listener);
    return () => this.off(event, listener);
  }

  once<K extends keyof Events>(event: K, listener: (payload: Events[K]) => void): () => void {
    const off = this.on(event, (payload) => {
      off();
      listener(payload);
    });
    return off;
  }

  off<K extends keyof Events>(event: K, listener: (payload: Events[K]) => void): void {
    const set = this.listeners.get(event);
    set?.delete(listener);
    if (set?.size === 0) this.listeners.delete(event);
  }

  emit<K extends keyof Events>(event: K, payload: Events[K]): void {
    // Copy so listeners can unsubscribe while being notified.
    for (const listener of [...(this.listeners.get(event) ?? [])]) {
      (listener as (payload: Events[K]) => void)(payload);
    }
  }

  listenerCount(event: keyof Events): number {
    return this.listeners.get(event)?.size ?? 0;
  }

  removeAllListeners(event?: keyof Events): void {
    if (event === undefined) this.listeners.clear();
    else this.listeners.delete(event);
  }
}
