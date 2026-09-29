export async function retry<T>(
  fn: (attempt: number) => Promise<T>,
  opts?: {
    retries?: number;
    baseDelayMs?: number;
    factor?: number;
    maxDelayMs?: number;
    shouldRetry?: (err: unknown, attempt: number) => boolean;
    sleep?: (ms: number) => Promise<void>;
  },
): Promise<T> {
  const retries = opts?.retries ?? 3;
  const baseDelayMs = opts?.baseDelayMs ?? 100;
  const factor = opts?.factor ?? 2;
  const maxDelayMs = opts?.maxDelayMs ?? 10000;
  const shouldRetry = opts?.shouldRetry ?? (() => true);
  const sleep = opts?.sleep ?? ((ms: number) => new Promise((resolve) => setTimeout(resolve, ms)));

  let lastError: unknown;

  for (let attempt = 1; attempt <= retries + 1; attempt++) {
    try {
      return await fn(attempt);
    } catch (error) {
      lastError = error;

      if (attempt <= retries && shouldRetry(error, attempt)) {
        const delayMs = Math.min(maxDelayMs, baseDelayMs * factor ** (attempt - 1));
        await sleep(delayMs);
      } else {
        throw error;
      }
    }
  }

  throw lastError;
}
