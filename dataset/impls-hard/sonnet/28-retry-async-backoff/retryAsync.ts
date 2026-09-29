export interface RetryOptions {
  /** Total number of attempts including the first one. */
  maxAttempts?: number;
  baseDelayMs?: number;
  factor?: number;
  maxDelayMs?: number;
  /** Randomise each delay between 50% and 100% of its nominal value. */
  jitter?: boolean;
  /** Return false to stop retrying for a given error. */
  shouldRetry?: (error: unknown, attempt: number) => boolean;
  signal?: AbortSignal;
}

function delay(ms: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) {
      reject(signal.reason);
      return;
    }
    const onAbort = () => {
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

/**
 * Run `operation` and retry it with exponentially growing delays when it rejects.
 * The last error is rethrown once all attempts are used up.
 */
export async function retryAsync<T>(
  operation: (attempt: number) => Promise<T>,
  options: RetryOptions = {},
): Promise<T> {
  const {
    maxAttempts = 4,
    baseDelayMs = 200,
    factor = 2,
    maxDelayMs = 10_000,
    jitter = true,
    shouldRetry = () => true,
    signal,
  } = options;
  if (maxAttempts < 1) throw new RangeError("maxAttempts must be at least 1");

  for (let attempt = 1; ; attempt++) {
    try {
      return await operation(attempt);
    } catch (error) {
      if (attempt >= maxAttempts || !shouldRetry(error, attempt)) throw error;
      let wait = Math.min(maxDelayMs, baseDelayMs * factor ** (attempt - 1));
      if (jitter) wait *= 0.5 + Math.random() / 2;
      await delay(wait, signal);
    }
  }
}
