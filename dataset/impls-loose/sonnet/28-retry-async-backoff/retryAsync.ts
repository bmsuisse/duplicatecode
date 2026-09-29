export interface RetryOptions {
  /** Total number of attempts including the first (default 5). */
  attempts?: number;
  baseDelayMs?: number;
  factor?: number;
  maxDelayMs?: number;
  /** Randomise each delay between 50% and 100% of its nominal value. */
  jitter?: boolean;
  /** Return false to stop retrying and rethrow immediately. */
  shouldRetry?: (error: unknown, attempt: number) => boolean;
  signal?: AbortSignal;
}

function sleep(ms: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) {
      reject(signal.reason);
      return;
    }
    const onAbort = (): void => {
      clearTimeout(timer);
      reject(signal?.reason);
    };
    const timer = setTimeout(() => {
      signal?.removeEventListener("abort", onAbort);
      resolve();
    }, ms);
    signal?.addEventListener("abort", onAbort, { once: true });
  });
}

/** Run `operation`, retrying on failure with exponentially growing delays. */
export async function retryAsync<T>(
  operation: (attempt: number) => Promise<T>,
  options: RetryOptions = {},
): Promise<T> {
  const {
    attempts = 5,
    baseDelayMs = 200,
    factor = 2,
    maxDelayMs = 30_000,
    jitter = false,
    shouldRetry = () => true,
    signal,
  } = options;
  if (attempts < 1) throw new RangeError("attempts must be at least 1");

  for (let attempt = 1; ; attempt++) {
    try {
      return await operation(attempt);
    } catch (error) {
      if (attempt >= attempts || !shouldRetry(error, attempt)) throw error;
      const nominal = Math.min(baseDelayMs * factor ** (attempt - 1), maxDelayMs);
      await sleep(jitter ? nominal * (0.5 + Math.random() / 2) : nominal, signal);
    }
  }
}
