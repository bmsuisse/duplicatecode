import { useCallback, useEffect, useState } from "react";

function read<T>(key: string, initial: T): T {
  try {
    const raw = window.localStorage.getItem(key);
    return raw === null ? initial : (JSON.parse(raw) as T);
  } catch {
    return initial;
  }
}

/** Like `useState`, but the value is persisted to localStorage as JSON and synced across tabs. */
export function useLocalStorageState<T>(
  key: string,
  initialValue: T,
): [T, (value: T | ((prev: T) => T)) => void, () => void] {
  const [state, setState] = useState<T>(() =>
    typeof window === "undefined" ? initialValue : read(key, initialValue),
  );

  useEffect(() => {
    setState(read(key, initialValue));
  }, [key]);

  useEffect(() => {
    const onStorage = (event: StorageEvent): void => {
      if (event.storageArea === window.localStorage && event.key === key) {
        setState(event.newValue === null ? initialValue : read(key, initialValue));
      }
    };
    window.addEventListener("storage", onStorage);
    return () => window.removeEventListener("storage", onStorage);
  }, [key]);

  const set = useCallback(
    (value: T | ((prev: T) => T)) => {
      setState((prev) => {
        const next = typeof value === "function" ? (value as (p: T) => T)(prev) : value;
        try {
          window.localStorage.setItem(key, JSON.stringify(next));
        } catch {
          // Storage full or unavailable: keep the in-memory value.
        }
        return next;
      });
    },
    [key],
  );

  const remove = useCallback(() => {
    try {
      window.localStorage.removeItem(key);
    } catch {
      // ignore
    }
    setState(initialValue);
  }, [key]);

  return [state, set, remove];
}
