export async function mapWithLimit<T, R>(
  items: T[],
  fn: (item: T) => Promise<R>,
  limit: number,
): Promise<R[]> {
  const results: R[] = [];
  const executing: Promise<any>[] = [];

  for (let i = 0; i < items.length; i++) {
    const promise = Promise.resolve(items[i])
      .then((item) => fn(item))
      .then((result) => {
        results[i] = result;
      });

    executing.push(promise);

    if (executing.length >= limit) {
      await Promise.race(executing);
      executing.splice(executing.indexOf(promise), 1);
    }
  }

  await Promise.all(executing);
  return results;
}
