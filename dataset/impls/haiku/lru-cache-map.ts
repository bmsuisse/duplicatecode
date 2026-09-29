export class LruCache<K, V> {
  private capacity: number;
  private cache: Map<K, V>;
  private onEvict?: (key: K, value: V) => void;

  constructor(capacity: number, onEvict?: (key: K, value: V) => void) {
    if (!Number.isInteger(capacity) || capacity < 1) {
      throw new RangeError("capacity must be a positive integer");
    }
    this.capacity = capacity;
    this.cache = new Map();
    this.onEvict = onEvict;
  }

  get(key: K): V | undefined {
    if (!this.cache.has(key)) {
      return undefined;
    }
    // Mark as recently used by removing and re-adding
    const value = this.cache.get(key);
    if (value !== undefined) {
      this.cache.delete(key);
      this.cache.set(key, value);
      return value;
    }
    return undefined;
  }

  set(key: K, value: V): this {
    if (this.cache.has(key)) {
      // Update existing key and mark as recently used
      this.cache.delete(key);
      this.cache.set(key, value);
    } else {
      // New key
      this.cache.set(key, value);

      // Evict if over capacity
      if (this.cache.size > this.capacity) {
        const oldestKey = this.cache.keys().next().value as K;
        const oldestValue = this.cache.get(oldestKey);
        if (oldestValue !== undefined) {
          this.cache.delete(oldestKey);
          this.onEvict?.(oldestKey, oldestValue);
        }
      }
    }
    return this;
  }

  has(key: K): boolean {
    return this.cache.has(key);
  }

  delete(key: K): boolean {
    return this.cache.delete(key);
  }

  get size(): number {
    return this.cache.size;
  }

  keys(): K[] {
    return Array.from(this.cache.keys());
  }
}
