import { useCallback, useState } from "react";

export function useLocalStorageState<T>(
  key: string,
  initial: T,
): [T, (next: T | ((prev: T) => T)) => void, () => void] {
  const [value, setValue] = useState<T>(() => {
    try {
      if (typeof window === "undefined") {
        return initial;
      }
      const stored = window.localStorage.getItem(key);
      if (stored === null) {
        return initial;
      }
      return JSON.parse(stored) as T;
    } catch {
      return initial;
    }
  });

  const setStoredValue = useCallback(
    (next: T | ((prev: T) => T)) => {
      setValue((prev) => {
        const newValue = typeof next === "function" ? (next as (prev: T) => T)(prev) : next;
        try {
          if (typeof window !== "undefined") {
            window.localStorage.setItem(key, JSON.stringify(newValue));
          }
        } catch {
          // Swallow write errors
        }
        return newValue;
      });
    },
    [key],
  );

  const remove = useCallback(() => {
    try {
      if (typeof window !== "undefined") {
        window.localStorage.removeItem(key);
      }
    } catch {
      // Swallow errors
    }
    setValue(initial);
  }, [key, initial]);

  return [value, setStoredValue, remove];
}
