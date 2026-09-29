export interface CsvOptions {
  delimiter?: string;
  quote?: string;
  /** Skip rows consisting of a single empty field (default true). */
  skipEmptyLines?: boolean;
}

/**
 * Parse CSV text into rows of fields (RFC 4180 style): quoted fields may contain
 * delimiters, newlines and doubled quotes. Handles \n, \r\n and \r line endings.
 */
export function parseCsv(text: string, options: CsvOptions = {}): string[][] {
  const { delimiter = ",", quote = '"', skipEmptyLines = true } = options;
  if (delimiter.length !== 1 || quote.length !== 1 || delimiter === quote) {
    throw new Error("delimiter and quote must be distinct single characters");
  }
  const input = text.startsWith("﻿") ? text.slice(1) : text;
  const rows: string[][] = [];
  let row: string[] = [];
  let field = "";
  let inQuotes = false;
  let sawContent = false;

  const endField = (): void => {
    row.push(field);
    field = "";
  };
  const endRow = (): void => {
    endField();
    if (!(skipEmptyLines && row.length === 1 && row[0] === "" && !sawContent)) rows.push(row);
    row = [];
    sawContent = false;
  };

  for (let i = 0; i < input.length; i++) {
    const ch = input[i];
    if (inQuotes) {
      if (ch === quote) {
        if (input[i + 1] === quote) {
          field += quote;
          i++;
        } else {
          inQuotes = false;
        }
      } else {
        field += ch;
      }
    } else if (ch === quote) {
      inQuotes = true;
      sawContent = true;
    } else if (ch === delimiter) {
      endField();
      sawContent = true;
    } else if (ch === "\n" || ch === "\r") {
      if (ch === "\r" && input[i + 1] === "\n") i++;
      endRow();
    } else {
      field += ch;
      sawContent = true;
    }
  }
  if (inQuotes) throw new Error("Unterminated quoted field");
  if (field !== "" || row.length > 0 || sawContent) endRow();
  return rows;
}
