export function debounce<A extends unknown[]>(
  fn: (...args: A) => void,
  waitMs: number,
  options?: { leading?: boolean; trailing?: boolean },
): ((...args: A) => void) & { cancel(): void; flush(): void } {
  const leading = options?.leading ?? false;
  const trailing = options?.trailing ?? true;

  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  let lastArgs: A | null = null;
  let pendingCall = false;
  let lastCallTime: number | null = null;

  function cancel() {
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
      timeoutId = null;
    }
    lastArgs = null;
    pendingCall = false;
    lastCallTime = null;
  }

  function flush() {
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
      timeoutId = null;
    }
    if (pendingCall && lastArgs !== null && trailing) {
      fn(...lastArgs);
    }
    lastArgs = null;
    pendingCall = false;
    lastCallTime = null;
  }

  function debounced(...args: A): void {
    lastArgs = args;

    if (timeoutId !== null) {
      clearTimeout(timeoutId);
    }

    const now = Date.now();
    const isFirstCall = lastCallTime === null;

    if (isFirstCall && leading) {
      fn(...args);
      pendingCall = true;
    } else {
      pendingCall = true;
    }

    lastCallTime = now;

    timeoutId = setTimeout(() => {
      if (trailing && pendingCall && lastArgs !== null) {
        fn(...lastArgs);
      }
      timeoutId = null;
      lastArgs = null;
      pendingCall = false;
      lastCallTime = null;
    }, waitMs);
  }

  debounced.cancel = cancel;
  debounced.flush = flush;

  return debounced;
}
