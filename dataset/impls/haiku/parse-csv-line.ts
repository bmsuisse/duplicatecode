export function parseCsv(text: string, opts?: { delimiter?: string; trim?: boolean }): string[][] {
  const delimiter = opts?.delimiter ?? ",";
  const shouldTrim = opts?.trim ?? false;

  if (!text) {
    return [];
  }

  const rows: string[][] = [];
  let currentRow: string[] = [];
  let currentField = "";
  let inQuotes = false;
  let i = 0;

  while (i < text.length) {
    const char = text[i];

    if (inQuotes) {
      if (char === '"') {
        if (i + 1 < text.length && text[i + 1] === '"') {
          // Escaped quote
          currentField += '"';
          i += 2;
        } else {
          // End of quoted field
          inQuotes = false;
          i++;
        }
      } else if (char === "\n" || char === "\r") {
        // Newline inside quotes is literal
        currentField += char;
        i++;
        if (char === "\r" && i < text.length && text[i] === "\n") {
          currentField += "\n";
          i++;
        }
      } else {
        currentField += char;
        i++;
      }
    } else {
      if (char === '"') {
        inQuotes = true;
        i++;
      } else if (char === delimiter) {
        currentRow.push(shouldTrim ? currentField.trim() : currentField);
        currentField = "";
        i++;
      } else if (char === "\r") {
        if (i + 1 < text.length && text[i + 1] === "\n") {
          i++;
        }
        // End of row
        currentRow.push(shouldTrim ? currentField.trim() : currentField);
        // Only add row if it has content
        if (currentRow.length > 0 && currentRow.some((f) => f !== "")) {
          rows.push(currentRow);
        }
        currentRow = [];
        currentField = "";
        i++;
      } else if (char === "\n") {
        // End of row
        currentRow.push(shouldTrim ? currentField.trim() : currentField);
        if (currentRow.length > 0 && currentRow.some((f) => f !== "")) {
          rows.push(currentRow);
        }
        currentRow = [];
        currentField = "";
        i++;
      } else {
        currentField += char;
        i++;
      }
    }
  }

  if (inQuotes) {
    throw new Error("Unterminated quote");
  }

  // Add final field and row
  if (currentField !== "" || currentRow.length > 0) {
    currentRow.push(shouldTrim ? currentField.trim() : currentField);
    if (currentRow.some((f) => f !== "")) {
      rows.push(currentRow);
    }
  }

  return rows;
}
