import { useEffect, useRef, useState } from "react";

export interface SearchBoxProps {
  onSearch: (query: string) => void;
  delayMs?: number;
  placeholder?: string;
  initialValue?: string;
  minLength?: number;
}

export function SearchBox({
  onSearch,
  delayMs = 300,
  placeholder,
  initialValue = "",
  minLength = 0,
}: SearchBoxProps) {
  const [text, setText] = useState(initialValue);
  const latestOnSearch = useRef(onSearch);
  const lastEmitted = useRef<string | null>(null);
  const skipNext = useRef(true);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  latestOnSearch.current = onSearch;

  const emit = (query: string) => {
    if (query !== lastEmitted.current) {
      lastEmitted.current = query;
      latestOnSearch.current(query);
    }
  };

  useEffect(() => {
    if (skipNext.current) {
      skipNext.current = false;
      return;
    }
    const query = text.trim();
    if (query !== "" && query.length < minLength) {
      return;
    }
    timer.current = setTimeout(() => emit(query), delayMs);
    return () => clearTimeout(timer.current);
  }, [text, delayMs, minLength]);

  const clear = () => {
    clearTimeout(timer.current);
    skipNext.current = true;
    setText("");
    emit("");
  };

  return (
    <div>
      <input
        type="search"
        aria-label="Search"
        placeholder={placeholder}
        value={text}
        onChange={(event: { target: { value: string } }) => setText(event.target.value)}
      />
      {text !== "" && (
        <button type="button" onClick={clear}>
          Clear
        </button>
      )}
    </div>
  );
}
