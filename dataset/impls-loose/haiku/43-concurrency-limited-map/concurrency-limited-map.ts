export async function concurrencyLimitedMap<T, R>(
  items: T[],
  fn: (item: T) => Promise<R>,
  limit: number,
): Promise<R[]> {
  const results: R[] = [];
  const queue = [...items];
  let running = 0;
  let index = 0;

  return new Promise((resolve, reject) => {
    const process = async () => {
      while (queue.length > 0 && running < limit) {
        running++;
        const item = queue.shift()!;
        const currentIndex = index++;

        try {
          const result = await fn(item);
          results[currentIndex] = result;
        } catch (error) {
          return reject(error);
        } finally {
          running--;
          if (queue.length > 0 || running > 0) {
            process();
          } else {
            resolve(results);
          }
        }
      }
    };

    process();
  });
}
