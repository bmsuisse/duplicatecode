export function chunk<T>(
  items: readonly T[],
  size: number,
  opts: { dropRemainder?: boolean } = {},
): T[][] {
  if (!Number.isInteger(size) || size < 1) {
    throw new RangeError("size must be a positive integer");
  }
  const chunks: T[][] = [];
  for (let start = 0; start < items.length; start += size) {
    const piece = items.slice(start, start + size);
    if (piece.length < size && opts.dropRemainder) {
      break;
    }
    chunks.push(piece);
  }
  return chunks;
}
