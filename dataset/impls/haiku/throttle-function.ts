export function throttle<A extends unknown[]>(
  fn: (...args: A) => void,
  intervalMs: number,
  options?: { leading?: boolean; trailing?: boolean },
): ((...args: A) => void) & { cancel(): void } {
  const leading = options?.leading ?? true;
  const trailing = options?.trailing ?? true;

  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  let lastArgs: A | null = null;
  let lastRunTime: number | null = null;
  let inThrottle = false;

  function cancel() {
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
      timeoutId = null;
    }
    lastArgs = null;
    lastRunTime = null;
    inThrottle = false;
  }

  function throttled(...args: A): void {
    const now = Date.now();

    if (lastRunTime === null) {
      // First call
      if (leading) {
        fn(...args);
        lastRunTime = now;
        inThrottle = true;

        if (trailing) {
          timeoutId = setTimeout(() => {
            if (lastArgs !== null) {
              fn(...lastArgs);
              lastRunTime = Date.now();
              lastArgs = null;
              timeoutId = setTimeout(() => {
                inThrottle = false;
                lastRunTime = null;
              }, intervalMs);
            } else {
              inThrottle = false;
              lastRunTime = null;
              timeoutId = null;
            }
          }, intervalMs);
        } else {
          timeoutId = setTimeout(() => {
            inThrottle = false;
            lastRunTime = null;
            timeoutId = null;
          }, intervalMs);
        }
      } else {
        // leading=false: just schedule for later
        lastArgs = args;
        inThrottle = true;
        timeoutId = setTimeout(() => {
          fn(...args);
          lastRunTime = Date.now();
          lastArgs = null;
          inThrottle = false;
          timeoutId = null;
        }, intervalMs);
      }
    } else if (inThrottle) {
      // In throttle window, save args for trailing call
      lastArgs = args;
    } else {
      // Window expired, run immediately if leading
      if (leading) {
        fn(...args);
        lastRunTime = now;
        inThrottle = true;
        if (trailing) {
          timeoutId = setTimeout(() => {
            if (lastArgs !== null) {
              fn(...lastArgs);
              lastRunTime = Date.now();
              lastArgs = null;
              timeoutId = setTimeout(() => {
                inThrottle = false;
                lastRunTime = null;
              }, intervalMs);
            } else {
              inThrottle = false;
              lastRunTime = null;
              timeoutId = null;
            }
          }, intervalMs);
        } else {
          timeoutId = setTimeout(() => {
            inThrottle = false;
            lastRunTime = null;
            timeoutId = null;
          }, intervalMs);
        }
      }
    }
  }

  throttled.cancel = cancel;

  return throttled;
}
