export async function mapLimit<T, R>(
  items: readonly T[],
  limit: number,
  fn: (item: T, index: number) => Promise<R>,
): Promise<R[]> {
  if (!Number.isInteger(limit) || limit < 1) {
    throw new RangeError("limit must be a positive integer");
  }

  if (items.length === 0) {
    return [];
  }

  const results: R[] = new Array(items.length);
  let nextIndex = 0;
  let running = 0;
  let errorOccurred = false;

  return new Promise((resolve, reject) => {
    const startWorker = async () => {
      while (nextIndex < items.length && !errorOccurred) {
        const index = nextIndex;
        const item = items[index];
        nextIndex++;
        running++;

        try {
          results[index] = await fn(item, index);
        } catch (error) {
          errorOccurred = true;

          reject(error);
          return;
        } finally {
          running--;
          if (nextIndex >= items.length && running === 0) {
            resolve(results);
          }
        }

        // Start next worker if there are items left and limit not reached
        if (nextIndex < items.length && running < limit) {
          startWorker();
        }
      }

      if (running === 0) {
        resolve(results);
      }
    };

    // Start initial workers
    const initialWorkers = Math.min(limit, items.length);
    for (let i = 0; i < initialWorkers; i++) {
      startWorker();
    }
  });
}
