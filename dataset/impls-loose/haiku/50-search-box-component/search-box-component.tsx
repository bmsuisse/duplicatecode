import { useState } from "react";
import { useDebouncedValue } from "./use-debounced-value";

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
  const [input, setInput] = useState("");
  const debouncedInput = useDebouncedValue(input, debounceDelay);

  React.useEffect(() => {
    onSearch(debouncedInput);
  }, [debouncedInput, onSearch]);

  const handleClear = () => {
    setInput("");
  };

  return (
    <div className="flex items-center gap-2">
      <input
        type="text"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        placeholder={placeholder}
        className="px-3 py-2 border border-gray-300 rounded"
      />
      {input && (
        <button type="button" onClick={handleClear} className="px-2 py-1 bg-gray-300 rounded">
          Clear
        </button>
      )}
    </div>
  );
}
