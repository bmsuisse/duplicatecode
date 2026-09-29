export class LruCache<K, V> {
  private readonly entries = new Map<K, V>();

  constructor(
    private readonly capacity: number,
    private readonly onEvict?: (key: K, value: V) => void,
  ) {
    if (!Number.isInteger(capacity) || capacity < 1) {
      throw new RangeError("capacity must be a positive integer");
    }
  }

  get(key: K): V | undefined {
    if (!this.entries.has(key)) {
      return undefined;
    }
    const value = this.entries.get(key) as V;
    this.entries.delete(key);
    this.entries.set(key, value);
    return value;
  }

  set(key: K, value: V): this {
    this.entries.delete(key);
    this.entries.set(key, value);
    if (this.entries.size > this.capacity) {
      for (const [oldKey, oldValue] of this.entries) {
        this.entries.delete(oldKey);
        this.onEvict?.(oldKey, oldValue);
        break;
      }
    }
    return this;
  }

  has(key: K): boolean {
    return this.entries.has(key);
  }

  delete(key: K): boolean {
    return this.entries.delete(key);
  }

  get size(): number {
    return this.entries.size;
  }

  keys(): K[] {
    return [...this.entries.keys()];
  }
}
