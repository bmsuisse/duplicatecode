export interface Debounced<A extends unknown[]> {
  (...args: A): void;
  /** Cancel a pending invocation. */
  cancel(): void;
  /** Run a pending invocation immediately. */
  flush(): void;
}

/**
 * Wrap `fn` so it only runs after `waitMs` milliseconds passed without a new call.
 * The latest arguments are used. With `leading`, the first call of a burst runs immediately.
 */
export function debounce<A extends unknown[]>(
  fn: (...args: A) => void,
  waitMs: number,
  options: { leading?: boolean } = {},
): Debounced<A> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pendingArgs: A | undefined;

  const run = () => {
    timer = undefined;
    if (pendingArgs) {
      const args = pendingArgs;
      pendingArgs = undefined;
      fn(...args);
    }
  };

  const debounced = (...args: A): void => {
    const startOfBurst = timer === undefined;
    if (timer !== undefined) clearTimeout(timer);
    if (options.leading && startOfBurst) {
      pendingArgs = undefined;
      fn(...args);
    } else {
      pendingArgs = args;
    }
    timer = setTimeout(run, waitMs);
  };

  debounced.cancel = () => {
    if (timer !== undefined) clearTimeout(timer);
    timer = undefined;
    pendingArgs = undefined;
  };

  debounced.flush = () => {
    if (timer === undefined) return;
    clearTimeout(timer);
    run();
  };

  return debounced;
}
