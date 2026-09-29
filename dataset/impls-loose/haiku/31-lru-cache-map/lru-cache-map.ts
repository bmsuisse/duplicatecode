export class LRUCache<K, V> {
  private cache: Map<K, V>;
  private accessOrder: K[];
  private capacity: number;

  constructor(capacity: number) {
    if (capacity <= 0) throw new Error("Capacity must be positive");
    this.capacity = capacity;
    this.cache = new Map();
    this.accessOrder = [];
  }

  get(key: K): V {
    if (!this.cache.has(key)) {
      throw new Error("Key not found");
    }
    this.accessOrder = this.accessOrder.filter((k) => k !== key);
    this.accessOrder.push(key);
    return this.cache.get(key)!;
  }

  set(key: K, value: V): void {
    if (this.cache.has(key)) {
      this.accessOrder = this.accessOrder.filter((k) => k !== key);
    } else if (this.cache.size >= this.capacity) {
      const lruKey = this.accessOrder.shift()!;
      this.cache.delete(lruKey);
    }

    this.cache.set(key, value);
    this.accessOrder.push(key);
  }

  has(key: K): boolean {
    return this.cache.has(key);
  }

  clear(): void {
    this.cache.clear();
    this.accessOrder = [];
  }
}
