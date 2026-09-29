export class LRUCache<K, V> {
  private cache: Map<K, V>;
  private capacity: number;

  constructor(capacity: number) {
    if (capacity <= 0) {
      throw new Error("Capacity must be positive");
    }
    this.capacity = capacity;
    this.cache = new Map();
  }

  get(key: K): V {
    const value = this.cache.get(key);
    if (value === undefined) {
      throw new Error("Key not found");
    }
    this.cache.delete(key);
    this.cache.set(key, value);
    return value;
  }

  put(key: K, value: V): void {
    if (this.cache.has(key)) {
      this.cache.delete(key);
    }
    this.cache.set(key, value);
    if (this.cache.size > this.capacity) {
      const firstKey = this.cache.keys().next().value;
      this.cache.delete(firstKey);
    }
  }

  size(): number {
    return this.cache.size;
  }
}
