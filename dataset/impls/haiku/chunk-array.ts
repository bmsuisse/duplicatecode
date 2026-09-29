export function chunk<T>(
  items: readonly T[],
  size: number,
  opts?: { dropRemainder?: boolean },
): T[][] {
  if (!Number.isInteger(size) || size < 1) {
    throw new RangeError("size must be a positive integer");
  }

  const dropRemainder = opts?.dropRemainder ?? false;
  const result: T[][] = [];

  for (let i = 0; i < items.length; i += size) {
    const chunkItems = items.slice(i, i + size);
    if (chunkItems.length === size || !dropRemainder) {
      result.push(Array.from(chunkItems));
    }
  }

  return result;
}
