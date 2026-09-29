type Registration<P> = { handler: (payload: P) => void; once: boolean };

export class Emitter<Events extends Record<string, unknown>> {
  private readonly registry = new Map<keyof Events, Registration<never>[]>();

  private register<K extends keyof Events>(
    event: K,
    handler: (payload: Events[K]) => void,
    once: boolean,
  ): () => void {
    const list = (this.registry.get(event) ?? []) as Registration<Events[K]>[];
    const entry: Registration<Events[K]> = { handler, once };
    list.push(entry);
    this.registry.set(event, list as Registration<never>[]);
    return () => {
      const current = this.registry.get(event) as Registration<Events[K]>[] | undefined;
      const index = current ? current.indexOf(entry) : -1;
      if (current && index >= 0) {
        current.splice(index, 1);
      }
    };
  }

  on<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): () => void {
    return this.register(event, handler, false);
  }

  once<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): () => void {
    return this.register(event, handler, true);
  }

  off<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): void {
    const list = this.registry.get(event) as Registration<Events[K]>[] | undefined;
    const index = list ? list.findIndex((entry) => entry.handler === handler) : -1;
    if (list && index >= 0) {
      list.splice(index, 1);
    }
  }

  emit<K extends keyof Events>(event: K, payload: Events[K]): number {
    const list = this.registry.get(event) as Registration<Events[K]>[] | undefined;
    if (!list) {
      return 0;
    }
    let invoked = 0;
    for (const entry of [...list]) {
      const index = list.indexOf(entry);
      if (index < 0) {
        continue;
      }
      if (entry.once) {
        list.splice(index, 1);
      }
      invoked++;
      entry.handler(payload);
    }
    return invoked;
  }

  listenerCount<K extends keyof Events>(event: K): number {
    return this.registry.get(event)?.length ?? 0;
  }
}
