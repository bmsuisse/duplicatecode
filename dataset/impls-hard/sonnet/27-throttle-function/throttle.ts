export interface Throttled<A extends unknown[]> {
  (...args: A): void;
  cancel(): void;
}

/**
 * Wrap `fn` so it runs at most once per `intervalMs`. The first call runs immediately;
 * calls made during the cooldown are collapsed into one trailing call with the latest arguments.
 */
export function throttle<A extends unknown[]>(
  fn: (...args: A) => void,
  intervalMs: number,
): Throttled<A> {
  let lastRun = Number.NEGATIVE_INFINITY;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let trailingArgs: A | undefined;

  const invoke = (args: A) => {
    lastRun = Date.now();
    fn(...args);
  };

  const throttled = (...args: A): void => {
    const remaining = intervalMs - (Date.now() - lastRun);
    if (remaining <= 0) {
      if (timer !== undefined) {
        clearTimeout(timer);
        timer = undefined;
      }
      trailingArgs = undefined;
      invoke(args);
      return;
    }
    trailingArgs = args;
    if (timer === undefined) {
      timer = setTimeout(() => {
        timer = undefined;
        const pending = trailingArgs;
        trailingArgs = undefined;
        if (pending) invoke(pending);
      }, remaining);
    }
  };

  throttled.cancel = () => {
    if (timer !== undefined) clearTimeout(timer);
    timer = undefined;
    trailingArgs = undefined;
  };

  return throttled;
}
