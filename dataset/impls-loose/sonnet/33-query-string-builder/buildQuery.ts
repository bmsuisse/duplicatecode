export type QueryValue = string | number | boolean | Date | null | undefined;
export type QueryParams = Record<string, QueryValue | readonly QueryValue[]>;

export interface QueryOptions {
  /** "repeat": a=1&a=2 (default); "comma": a=1,2; "brackets": a[]=1&a[]=2 */
  arrayFormat?: "repeat" | "comma" | "brackets";
  /** Also skip empty strings (default true). */
  skipEmptyStrings?: boolean;
}

function isEmpty(value: QueryValue, skipEmptyStrings: boolean): boolean {
  return (
    value === null ||
    value === undefined ||
    (typeof value === "number" && Number.isNaN(value)) ||
    (skipEmptyStrings && value === "")
  );
}

function stringify(value: Exclude<QueryValue, null | undefined>): string {
  return value instanceof Date ? value.toISOString() : String(value);
}

/** Build a URL query string (without leading "?") from an object of params. */
export function buildQuery(params: QueryParams, options: QueryOptions = {}): string {
  const { arrayFormat = "repeat", skipEmptyStrings = true } = options;
  const enc = encodeURIComponent;
  const parts: string[] = [];
  for (const [key, raw] of Object.entries(params)) {
    if (Array.isArray(raw)) {
      const values = (raw as readonly QueryValue[]).filter(
        (v): v is Exclude<QueryValue, null | undefined> => !isEmpty(v, skipEmptyStrings),
      );
      if (values.length === 0) continue;
      if (arrayFormat === "comma") {
        parts.push(`${enc(key)}=${values.map((v) => enc(stringify(v))).join(",")}`);
      } else {
        const name = arrayFormat === "brackets" ? `${enc(key)}[]` : enc(key);
        for (const v of values) parts.push(`${name}=${enc(stringify(v))}`);
      }
    } else {
      const value = raw as QueryValue;
      if (isEmpty(value, skipEmptyStrings)) continue;
      parts.push(`${enc(key)}=${enc(stringify(value as Exclude<QueryValue, null | undefined>))}`);
    }
  }
  return parts.join("&");
}
