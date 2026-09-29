import {
  type Dispatch,
  type SetStateAction,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

function readStored<T>(key: string, fallback: T): T {
  try {
    const raw = window.localStorage.getItem(key);
    return raw === null ? fallback : (JSON.parse(raw) as T);
  } catch {
    return fallback;
  }
}

/**
 * Like `useState`, but the value is persisted as JSON in localStorage under `key`,
 * restored on mount and kept in sync across browser tabs. Storage errors (private
 * mode, quota, invalid JSON) fall back to plain in-memory state.
 */
export function useLocalStorageState<T>(
  key: string,
  initialValue: T,
): [T, Dispatch<SetStateAction<T>>, () => void] {
  const initialRef = useRef(initialValue);
  const [value, setValue] = useState<T>(() => readStored(key, initialRef.current));

  // Re-read when the key changes.
  useEffect(() => {
    setValue(readStored(key, initialRef.current));
  }, [key]);

  useEffect(() => {
    try {
      window.localStorage.setItem(key, JSON.stringify(value));
    } catch {
      // Persisting is best effort.
    }
  }, [key, value]);

  useEffect(() => {
    const onStorage = (event: StorageEvent) => {
      if (event.key !== key) return;
      setValue(event.newValue === null ? initialRef.current : readStored(key, initialRef.current));
    };
    window.addEventListener("storage", onStorage);
    return () => window.removeEventListener("storage", onStorage);
  }, [key]);

  const reset = useCallback(() => {
    try {
      window.localStorage.removeItem(key);
    } catch {
      // Ignore storage errors.
    }
    setValue(initialRef.current);
  }, [key]);

  return [value, setValue, reset];
}
