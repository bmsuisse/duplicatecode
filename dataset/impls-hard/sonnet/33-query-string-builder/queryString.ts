type Primitive = string | number | boolean | Date;
export type QueryValue = Primitive | null | undefined | ReadonlyArray<Primitive | null | undefined>;

function stringify(value: Primitive): string {
  return value instanceof Date ? value.toISOString() : String(value);
}

export interface QueryStringOptions {
  /** How arrays are written: repeated keys (`a=1&a=2`) or comma separated (`a=1,2`). */
  arrayFormat?: "repeat" | "comma";
  /** Prepend "?" when the result is not empty. */
  addPrefix?: boolean;
}

/** Build a percent-encoded query string. null/undefined/empty-string values are omitted. */
export function buildQueryString(
  params: Readonly<Record<string, QueryValue>>,
  options: QueryStringOptions = {},
): string {
  const { arrayFormat = "repeat", addPrefix = false } = options;
  const parts: string[] = [];
  for (const [key, raw] of Object.entries(params)) {
    const values = (Array.isArray(raw) ? raw : [raw]) as ReadonlyArray<
      Primitive | null | undefined
    >;
    const present = values
      .filter((v): v is Primitive => v !== null && v !== undefined)
      .map(stringify)
      .filter((v) => v !== "");
    if (present.length === 0) continue;
    const name = encodeURIComponent(key);
    if (arrayFormat === "comma") {
      parts.push(`${name}=${present.map(encodeURIComponent).join(",")}`);
    } else {
      for (const value of present) parts.push(`${name}=${encodeURIComponent(value)}`);
    }
  }
  const query = parts.join("&");
  return addPrefix && query ? `?${query}` : query;
}
