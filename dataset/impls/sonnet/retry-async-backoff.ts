export async function retry<T>(
  fn: (attempt: number) => Promise<T>,
  opts: {
    retries?: number;
    baseDelayMs?: number;
    factor?: number;
    maxDelayMs?: number;
    shouldRetry?: (err: unknown, attempt: number) => boolean;
    sleep?: (ms: number) => Promise<void>;
  } = {},
): Promise<T> {
  const {
    retries = 3,
    baseDelayMs = 100,
    factor = 2,
    maxDelayMs = 10000,
    shouldRetry = () => true,
    sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms)),
  } = opts;

  for (let attempt = 1; ; attempt++) {
    try {
      return await fn(attempt);
    } catch (err) {
      if (attempt > retries || !shouldRetry(err, attempt)) {
        throw err;
      }
      await sleep(Math.min(maxDelayMs, baseDelayMs * factor ** (attempt - 1)));
    }
  }
}
