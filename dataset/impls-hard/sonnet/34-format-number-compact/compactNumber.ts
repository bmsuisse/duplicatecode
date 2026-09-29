const SUFFIXES = ["", "K", "M", "B", "T"] as const;

/**
 * Shorten large numbers: 1234567 -> "1.2M", 950 -> "950", 15000 -> "15K".
 * At most one decimal place; a trailing ".0" is dropped.
 */
export function formatCompactNumber(value: number): string {
  if (!Number.isFinite(value)) return String(value);
  const sign = value < 0 ? "-" : "";
  let scaled = Math.abs(value);
  let index = 0;
  while (scaled >= 1000 && index < SUFFIXES.length - 1) {
    scaled /= 1000;
    index++;
  }
  let rounded = Math.round(scaled * 10) / 10;
  // Rounding may push e.g. 999.95K to 1000K; move up one unit in that case.
  if (rounded >= 1000 && index < SUFFIXES.length - 1) {
    rounded = Math.round(rounded / 100) / 10;
    index++;
  }
  const text = index === 0 ? String(Math.round(scaled)) : rounded.toFixed(1).replace(/\.0$/, "");
  return `${sign}${text}${SUFFIXES[index]}`;
}
