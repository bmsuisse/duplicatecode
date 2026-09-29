/** Generic LRU cache; the Map's insertion order tracks recency. */
export class LruCache<K, V> {
  private readonly entries = new Map<K, V>();

  constructor(readonly maxSize: number) {
    if (!Number.isInteger(maxSize) || maxSize < 1) {
      throw new RangeError("maxSize must be a positive integer");
    }
  }

  get size(): number {
    return this.entries.size;
  }

  get(key: K): V | undefined {
    if (!this.entries.has(key)) return undefined;
    const value = this.entries.get(key) as V;
    this.entries.delete(key);
    this.entries.set(key, value);
    return value;
  }

  /** Check for a key without affecting recency. */
  has(key: K): boolean {
    return this.entries.has(key);
  }

  set(key: K, value: V): this {
    this.entries.delete(key);
    this.entries.set(key, value);
    if (this.entries.size > this.maxSize) {
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

  /** Iterate keys from least to most recently used. */
  keys(): IterableIterator<K> {
    return this.entries.keys();
  }
}
