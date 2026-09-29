/** Fixed-size cache that evicts the least recently used entry. Relies on Map insertion order. */
export class LruCache<K, V> {
  private readonly entries = new Map<K, V>();

  constructor(readonly capacity: number) {
    if (!Number.isInteger(capacity) || capacity < 1) {
      throw new RangeError("capacity must be a positive integer");
    }
  }

  get size(): number {
    return this.entries.size;
  }

  has(key: K): boolean {
    return this.entries.has(key);
  }

  /** Read a value and mark it as most recently used. */
  get(key: K): V | undefined {
    if (!this.entries.has(key)) return undefined;
    const value = this.entries.get(key) as V;
    this.entries.delete(key);
    this.entries.set(key, value);
    return value;
  }

  /** Read a value without changing its recency. */
  peek(key: K): V | undefined {
    return this.entries.get(key);
  }

  set(key: K, value: V): this {
    this.entries.delete(key);
    this.entries.set(key, value);
    if (this.entries.size > this.capacity) {
      const oldest = this.entries.keys().next();
      if (!oldest.done) this.entries.delete(oldest.value);
    }
    return this;
  }

  delete(key: K): boolean {
    return this.entries.delete(key);
  }

  clear(): void {
    this.entries.clear();
  }

  /** Keys ordered from least to most recently used. */
  keys(): K[] {
    return [...this.entries.keys()];
  }
}
