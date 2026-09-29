export interface CsvOptions {
  delimiter?: string;
  quote?: string;
}

/**
 * Parse CSV text into rows of fields (RFC 4180 style). Quoted fields may contain the
 * delimiter, line breaks and doubled quotes. Both \n and \r\n end a row.
 */
export function parseCsv(text: string, options: CsvOptions = {}): string[][] {
  const { delimiter = ",", quote = '"' } = options;
  if (delimiter.length !== 1 || quote.length !== 1 || delimiter === quote) {
    throw new Error("delimiter and quote must be distinct single characters");
  }
  const rows: string[][] = [];
  let row: string[] = [];
  let field = "";
  let inQuotes = false;
  let fieldStarted = false;

  const endField = () => {
    row.push(field);
    field = "";
    fieldStarted = false;
  };
  const endRow = () => {
    endField();
    rows.push(row);
    row = [];
  };

  for (let i = 0; i < text.length; i++) {
    const char = text.charAt(i);
    if (inQuotes) {
      if (char === quote) {
        if (text.charAt(i + 1) === quote) {
          field += quote;
          i++;
        } else {
          inQuotes = false;
        }
      } else {
        field += char;
      }
    } else if (char === quote && field === "") {
      inQuotes = true;
      fieldStarted = true;
    } else if (char === delimiter) {
      endField();
    } else if (char === "\n" || char === "\r") {
      if (char === "\r" && text.charAt(i + 1) === "\n") i++;
      endRow();
    } else {
      field += char;
      fieldStarted = true;
    }
  }
  if (inQuotes) throw new Error("Unterminated quoted field");
  if (fieldStarted || field !== "" || row.length > 0) endRow();
  return rows;
}
