export type EventMap = Record<string, any>;

export class EventEmitter<TEvents extends EventMap = Record<string, any>> {
  private listeners: Map<keyof TEvents, Set<Function>> = new Map();

  on<K extends keyof TEvents>(event: K, listener: (data: TEvents[K]) => void): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)?.add(listener);
  }

  off<K extends keyof TEvents>(event: K, listener: (data: TEvents[K]) => void): void {
    this.listeners.get(event)?.delete(listener);
  }

  emit<K extends keyof TEvents>(event: K, data: TEvents[K]): void {
    const handlers = this.listeners.get(event);
    if (handlers) {
      for (const listener of handlers) {
        listener(data);
      }
    }
  }
}
