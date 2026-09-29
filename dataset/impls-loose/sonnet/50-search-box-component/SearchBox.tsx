import { useEffect, useRef, useState } from "react";

export interface SearchBoxProps {
  /** Called with the trimmed query once typing pauses, and immediately on clear. */
  onSearch: (query: string) => void;
  delayMs?: number;
  placeholder?: string;
  initialValue?: string;
}

export function SearchBox({
  onSearch,
  delayMs = 300,
  placeholder = "Search…",
  initialValue = "",
}: SearchBoxProps) {
  const [value, setValue] = useState(initialValue);
  const lastSent = useRef(initialValue.trim());
  const onSearchRef = useRef(onSearch);
  onSearchRef.current = onSearch;

  useEffect(() => {
    const query = value.trim();
    if (query === lastSent.current) return;
    const timer = setTimeout(() => {
      lastSent.current = query;
      onSearchRef.current(query);
    }, delayMs);
    return () => clearTimeout(timer);
  }, [value, delayMs]);

  const clear = () => {
    setValue("");
    if (lastSent.current !== "") {
      lastSent.current = "";
      onSearchRef.current("");
    }
  };

  return (
    <search>
      <input
        type="search"
        value={value}
        placeholder={placeholder}
        aria-label={placeholder}
        onChange={(event) => setValue(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Escape") clear();
        }}
      />
      {value !== "" && (
        <button type="button" aria-label="Clear search" onClick={clear}>
          ×
        </button>
      )}
    </search>
  );
}
