export function buildQuery(
  params: Record<string, unknown>,
  opts?: { arrayFormat?: "repeat" | "comma" | "brackets"; prefix?: boolean },
): string {
  const arrayFormat = opts?.arrayFormat ?? "repeat";
  const prefix = opts?.prefix ?? false;

  const pairs: string[] = [];

  for (const [key, value] of Object.entries(params)) {
    if (value === undefined || value === null) {
      continue;
    }

    const encodedKey = encodeURIComponent(key);

    if (typeof value === "boolean") {
      pairs.push(`${encodedKey}=${encodeURIComponent(String(value).toLowerCase())}`);
    } else if (value instanceof Date) {
      pairs.push(`${encodedKey}=${encodeURIComponent(value.toISOString())}`);
    } else if (Array.isArray(value)) {
      if (value.length === 0) {
        continue;
      }

      if (arrayFormat === "repeat") {
        for (const item of value) {
          pairs.push(`${encodedKey}=${encodeURIComponent(String(item))}`);
        }
      } else if (arrayFormat === "comma") {
        const encoded = value.map((item) => encodeURIComponent(String(item))).join(",");
        pairs.push(`${encodedKey}=${encoded}`);
      } else if (arrayFormat === "brackets") {
        for (const item of value) {
          pairs.push(`${encodedKey}[]=${encodeURIComponent(String(item))}`);
        }
      }
    } else if (typeof value === "object") {
      throw new TypeError("Nested objects are not supported");
    } else {
      pairs.push(`${encodedKey}=${encodeURIComponent(String(value))}`);
    }
  }

  const result = pairs.join("&");
  return result.length > 0 && prefix ? `?${result}` : result;
}
