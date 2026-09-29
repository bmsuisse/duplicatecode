export async function retryAsync<T>(
  fn: () => Promise<T>,
  maxAttempts: number = 3,
  initialDelay: number = 1000,
  backoffFactor: number = 2,
): Promise<T> {
  let lastError: Error | null = null;

  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      if (attempt < maxAttempts - 1) {
        const delay = initialDelay * backoffFactor ** attempt;
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
    }
  }

  if (lastError) {
    throw lastError;
  }
  throw new Error("Retry failed");
}
