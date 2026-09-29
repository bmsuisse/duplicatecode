type EventMap = Record<string, any>;

export class TypedEventEmitter<Events extends EventMap> {
  private listeners: Map<keyof Events, Set<(data: any) => void>> = new Map();

  on<K extends keyof Events>(event: K, listener: (data: Events[K]) => void): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(listener);
  }

  off<K extends keyof Events>(event: K, listener: (data: Events[K]) => void): void {
    this.listeners.get(event)?.delete(listener);
  }

  emit<K extends keyof Events>(event: K, data: Events[K]): void {
    const eventListeners = this.listeners.get(event);
    if (eventListeners) {
      eventListeners.forEach((listener) => {
        listener(data);
      });
    }
  }

  once<K extends keyof Events>(event: K, listener: (data: Events[K]) => void): void {
    const wrapper = (data: Events[K]) => {
      listener(data);
      this.off(event, wrapper);
    };
    this.on(event, wrapper);
  }
}
