export interface ThrottledFunction<Args extends unknown[]> {
  (...args: Args): void;
  cancel(): void;
}

export interface ThrottleOptions {
  /** Run on the first call of a window (default true). */
  leading?: boolean;
  /** Run once more at the end of the window with the latest args (default true). */
  trailing?: boolean;
}

/** Ensure `fn` runs at most once every `intervalMs`. */
export function throttle<Args extends unknown[]>(
  fn: (...args: Args) => void,
  intervalMs: number,
  { leading = true, trailing = true }: ThrottleOptions = {},
): ThrottledFunction<Args> {
  let lastRun = Number.NEGATIVE_INFINITY;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let trailingArgs: Args | undefined;

  const run = (args: Args): void => {
    lastRun = Date.now();
    fn(...args);
  };

  const throttled = (...args: Args): void => {
    const now = Date.now();
    if (!leading && lastRun === Number.NEGATIVE_INFINITY) lastRun = now;
    const remaining = intervalMs - (now - lastRun);
    if (remaining <= 0) {
      if (timer !== undefined) {
        clearTimeout(timer);
        timer = undefined;
      }
      trailingArgs = undefined;
      run(args);
    } else if (trailing) {
      trailingArgs = args;
      timer ??= setTimeout(() => {
        timer = undefined;
        const pending = trailingArgs;
        trailingArgs = undefined;
        if (pending) run(pending);
      }, remaining);
    }
  };

  throttled.cancel = (): void => {
    if (timer !== undefined) clearTimeout(timer);
    timer = undefined;
    trailingArgs = undefined;
    lastRun = Number.NEGATIVE_INFINITY;
  };
  return throttled;
}
