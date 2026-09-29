export function debounce<A extends unknown[]>(
  fn: (...args: A) => void,
  waitMs: number,
  options: { leading?: boolean; trailing?: boolean } = {},
): ((...args: A) => void) & { cancel(): void; flush(): void } {
  const { leading = false, trailing = true } = options;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pendingArgs: A | undefined;

  const finish = () => {
    timer = undefined;
    const args = pendingArgs;
    pendingArgs = undefined;
    if (trailing && args) {
      fn(...args);
    }
  };

  const debounced = (...args: A): void => {
    const startsBurst = timer === undefined;
    if (timer !== undefined) {
      clearTimeout(timer);
    }
    if (startsBurst && leading) {
      fn(...args);
    } else {
      pendingArgs = args;
    }
    timer = setTimeout(finish, waitMs);
  };

  debounced.cancel = (): void => {
    if (timer !== undefined) {
      clearTimeout(timer);
    }
    timer = undefined;
    pendingArgs = undefined;
  };

  debounced.flush = (): void => {
    if (timer === undefined) {
      return;
    }
    clearTimeout(timer);
    finish();
  };

  return debounced;
}
