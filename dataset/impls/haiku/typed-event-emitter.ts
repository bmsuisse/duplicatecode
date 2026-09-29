export class Emitter<Events extends Record<string, unknown>> {
  private handlers: Map<string, Array<{ handler: (payload: unknown) => void; once: boolean }>> =
    new Map();

  on<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): () => void {
    const eventKey = String(event);
    if (!this.handlers.has(eventKey)) {
      this.handlers.set(eventKey, []);
    }
    const handlersArray = this.handlers.get(eventKey);
    if (handlersArray) {
      handlersArray.push({
        handler: handler as (payload: unknown) => void,
        once: false,
      });
    }

    return () => {
      this.off(event, handler);
    };
  }

  once<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): () => void {
    const eventKey = String(event);
    if (!this.handlers.has(eventKey)) {
      this.handlers.set(eventKey, []);
    }
    const handlersArray = this.handlers.get(eventKey);
    if (handlersArray) {
      handlersArray.push({
        handler: handler as (payload: unknown) => void,
        once: true,
      });
    }

    return () => {
      this.off(event, handler);
    };
  }

  off<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): void {
    const eventKey = String(event);
    const handlers = this.handlers.get(eventKey);
    if (!handlers) {
      return;
    }

    const index = handlers.findIndex((h) => h.handler === handler);
    if (index !== -1) {
      handlers.splice(index, 1);
    }
  }

  emit<K extends keyof Events>(event: K, payload: Events[K]): number {
    const eventKey = String(event);
    const handlers = this.handlers.get(eventKey);
    if (!handlers || handlers.length === 0) {
      return 0;
    }

    let count = 0;
    // Make a copy of the handlers array to avoid issues with handlers added/removed during emit
    const handlersCopy = [...handlers];

    for (const { handler, once } of handlersCopy) {
      // Check if handler is still in the list (might have been removed)
      const stillExists = handlers.some((h) => h.handler === handler);
      if (!stillExists) {
        continue;
      }

      if (once) {
        this.off(event, handler as (payload: Events[K]) => void);
      }

      handler(payload);
      count++;
    }

    return count;
  }

  listenerCount<K extends keyof Events>(event: K): number {
    const eventKey = String(event);
    return this.handlers.get(eventKey)?.length ?? 0;
  }
}
