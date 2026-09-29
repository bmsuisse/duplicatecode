export function throttle<A extends unknown[]>(
  fn: (...args: A) => void,
  intervalMs: number,
  options: { leading?: boolean; trailing?: boolean } = {},
): ((...args: A) => void) & { cancel(): void } {
  const { leading = true, trailing = true } = options;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pendingArgs: A | undefined;

  const openWindow = (): void => {
    timer = setTimeout(endWindow, intervalMs);
  };

  function endWindow(): void {
    timer = undefined;
    const args = pendingArgs;
    pendingArgs = undefined;
    if (trailing && args) {
      fn(...args);
      openWindow();
    }
  }

  const throttled = (...args: A): void => {
    if (timer !== undefined) {
      pendingArgs = args;
      return;
    }
    if (leading) {
      fn(...args);
    } else {
      pendingArgs = args;
    }
    openWindow();
  };

  throttled.cancel = (): void => {
    if (timer !== undefined) {
      clearTimeout(timer);
    }
    timer = undefined;
    pendingArgs = undefined;
  };

  return throttled;
}
