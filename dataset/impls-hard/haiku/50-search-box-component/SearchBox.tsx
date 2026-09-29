import { useCallback, useEffect, useState } from "react";
import { useDebouncedValue } from "./useDebouncedValue";

interface SearchBoxProps {
  onSearch: (query: string) => void;
  placeholder?: string;
  debounceDelay?: number;
}

export function SearchBox({
  onSearch,
  placeholder = "Search...",
  debounceDelay = 300,
}: SearchBoxProps) {
  const [value, setValue] = useState("");
  const debouncedValue = useDebouncedValue(value, debounceDelay);

  const handleClear = useCallback(() => {
    setValue("");
  }, []);

  useEffect(() => {
    onSearch(debouncedValue);
  }, [debouncedValue, onSearch]);

  return (
    <div className="flex gap-2">
      <input
        type="text"
        value={value}
        onChange={(e) => setValue(e.target.value)}
        placeholder={placeholder}
        className="flex-1 px-3 py-2 border border-gray-300 rounded"
      />
      {value && (
        <button
          type="button"
          onClick={handleClear}
          className="px-3 py-2 bg-gray-300 rounded hover:bg-gray-400"
        >
          Clear
        </button>
      )}
    </div>
  );
}
