export function parseCsv(
  text: string,
  opts: { delimiter?: string; trim?: boolean } = {},
): string[][] {
  const { delimiter = ",", trim = false } = opts;
  const rows: string[][] = [];
  let row: string[] = [];
  let field = "";
  let quoted = false;
  let inQuotes = false;
  let sawContent = false;

  const endField = () => {
    row.push(trim && !quoted ? field.trim() : field);
    field = "";
    quoted = false;
  };
  const endRow = () => {
    if (sawContent) {
      endField();
      rows.push(row);
    }
    row = [];
    field = "";
    quoted = false;
    sawContent = false;
  };

  for (let i = 0; i < text.length; i++) {
    const char = text[i];
    if (inQuotes) {
      if (char === '"' && text[i + 1] === '"') {
        field += '"';
        i++;
      } else if (char === '"') {
        inQuotes = false;
      } else {
        field += char;
      }
    } else if (char === '"' && field.trim() === "" && !quoted) {
      field = "";
      inQuotes = true;
      quoted = true;
      sawContent = true;
    } else if (char === delimiter) {
      endField();
      sawContent = true;
    } else if (char === "\n" || char === "\r") {
      if (char === "\r" && text[i + 1] === "\n") {
        i++;
      }
      endRow();
    } else {
      field += char;
      sawContent = true;
    }
  }
  if (inQuotes) {
    throw new Error("Unterminated quoted field");
  }
  endRow();
  return rows;
}
