import { useEffect, useRef, useState } from "react";

export interface SearchBoxProps {
  onSearch: (query: string) => void;
  delayMs?: number;
  placeholder?: string;
  initialValue?: string;
  minLength?: number;
}

export function SearchBox(props: SearchBoxProps) {
  const { onSearch, delayMs = 300, placeholder, initialValue = "", minLength = 0 } = props;

  const [inputValue, setInputValue] = useState(initialValue);
  const [lastEmittedValue, setLastEmittedValue] = useState<string | null>(null);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);
  const onSearchRef = useRef(onSearch);

  // Keep ref up to date
  useEffect(() => {
    onSearchRef.current = onSearch;
  }, [onSearch]);

  // Handle input change
  const handleInputChange = (e: { target: { value: string } }) => {
    const newValue = e.target.value;
    setInputValue(newValue);

    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }

    timeoutRef.current = setTimeout(() => {
      const trimmedValue = newValue.trim();

      // Check if we should call onSearch
      if (trimmedValue.length === 0 || trimmedValue.length >= minLength) {
        if (trimmedValue !== lastEmittedValue) {
          setLastEmittedValue(trimmedValue);
          onSearchRef.current(trimmedValue);
        }
      }
    }, delayMs);
  };

  // Handle clear button
  const handleClear = () => {
    setInputValue("");
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    setLastEmittedValue("");
    onSearchRef.current("");
  };

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  return (
    <div>
      <input
        type="search"
        aria-label="Search"
        placeholder={placeholder}
        value={inputValue}
        onChange={handleInputChange}
      />
      {inputValue && (
        <button type="button" onClick={handleClear}>
          Clear
        </button>
      )}
    </div>
  );
}
