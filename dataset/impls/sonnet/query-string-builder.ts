function encodeValue(value: unknown): string {
  if (value instanceof Date) {
    return encodeURIComponent(value.toISOString());
  }
  if (typeof value === "object" && value !== null) {
    throw new TypeError("nested objects are not supported");
  }
  return encodeURIComponent(typeof value === "boolean" ? String(value) : String(value));
}

export function buildQuery(
  params: Record<string, unknown>,
  opts: { arrayFormat?: "repeat" | "comma" | "brackets"; prefix?: boolean } = {},
): string {
  const { arrayFormat = "repeat", prefix = false } = opts;
  const parts: string[] = [];
  for (const [key, value] of Object.entries(params)) {
    if (value === undefined || value === null) {
      continue;
    }
    const name = encodeURIComponent(key);
    if (Array.isArray(value)) {
      if (value.length === 0) {
        continue;
      }
      const encoded = value.map(encodeValue);
      if (arrayFormat === "comma") {
        parts.push(`${name}=${encoded.join(",")}`);
      } else {
        const label = arrayFormat === "brackets" ? `${name}[]` : name;
        for (const item of encoded) {
          parts.push(`${label}=${item}`);
        }
      }
    } else {
      parts.push(`${name}=${encodeValue(value)}`);
    }
  }
  const query = parts.join("&");
  return prefix && query ? `?${query}` : query;
}
