import { useEffect, useState } from "react";

export function useDebouncedValue<T>(value: T, delayMs: number): T {
  const [debounced, setDebounced] = useState<T>(value);

  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), Math.max(delayMs, 0));
    return () => clearTimeout(timer);
  }, [value, delayMs]);

  return debounced;
}
