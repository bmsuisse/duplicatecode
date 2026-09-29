export interface DebouncedFunction<Args extends unknown[]> {
  (...args: Args): void;
  /** Drop any pending call. */
  cancel(): void;
  /** Run a pending call immediately. */
  flush(): void;
}

export interface DebounceOptions {
  /** Also fire on the first call of a burst. */
  leading?: boolean;
  /** Fire after the burst ends (default true). */
  trailing?: boolean;
}

/** Delay calling `fn` until `waitMs` has passed without further calls. */
export function debounce<Args extends unknown[]>(
  fn: (...args: Args) => void,
  waitMs: number,
  { leading = false, trailing = true }: DebounceOptions = {},
): DebouncedFunction<Args> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pendingArgs: Args | undefined;

  const invokePending = (): void => {
    if (pendingArgs) {
      const args = pendingArgs;
      pendingArgs = undefined;
      fn(...args);
    }
  };

  const debounced = (...args: Args): void => {
    const startingBurst = timer === undefined;
    if (timer !== undefined) clearTimeout(timer);
    if (startingBurst && leading) {
      fn(...args);
    } else if (trailing) {
      pendingArgs = args;
    }
    timer = setTimeout(() => {
      timer = undefined;
      invokePending();
    }, waitMs);
  };

  debounced.cancel = (): void => {
    if (timer !== undefined) clearTimeout(timer);
    timer = undefined;
    pendingArgs = undefined;
  };
  debounced.flush = (): void => {
    if (timer !== undefined) clearTimeout(timer);
    timer = undefined;
    invokePending();
  };
  return debounced;
}
