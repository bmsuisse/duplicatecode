import { useEffect, useRef, useState } from "react";

export interface SearchBoxProps {
  /** Called with the trimmed text once the user paused typing (or cleared the box). */
  onSearch: (query: string) => void;
  delayMs?: number;
  placeholder?: string;
  initialValue?: string;
  label?: string;
}

/** Search input that reports changes after a typing pause and offers a clear button. */
export function SearchBox({
  onSearch,
  delayMs = 300,
  placeholder = "Search...",
  initialValue = "",
  label = "Search",
}: SearchBoxProps) {
  const [text, setText] = useState(initialValue);
  const onSearchRef = useRef(onSearch);
  const lastReported = useRef(initialValue.trim());
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    onSearchRef.current = onSearch;
  }, [onSearch]);

  useEffect(() => {
    const query = text.trim();
    if (query === lastReported.current) return;
    const timer = setTimeout(() => {
      lastReported.current = query;
      onSearchRef.current(query);
    }, delayMs);
    return () => clearTimeout(timer);
  }, [text, delayMs]);

  const clear = () => {
    setText("");
    inputRef.current?.focus();
  };

  return (
    <div style={{ display: "inline-flex", alignItems: "center", gap: 4 }}>
      <input
        ref={inputRef}
        type="search"
        aria-label={label}
        placeholder={placeholder}
        value={text}
        onChange={(event) => setText(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Escape") clear();
        }}
      />
      {text !== "" && (
        <button type="button" aria-label="Clear search" onClick={clear}>
          x
        </button>
      )}
    </div>
  );
}
