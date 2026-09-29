const UNITS: ReadonlyArray<readonly [Intl.RelativeTimeFormatUnit, number]> = [
  ["year", 365 * 24 * 3600],
  ["month", 30 * 24 * 3600],
  ["week", 7 * 24 * 3600],
  ["day", 24 * 3600],
  ["hour", 3600],
  ["minute", 60],
];

/**
 * Describe how far `date` is from `now`, e.g. "5 minutes ago", "in 3 days", "just now".
 * Uses Intl.RelativeTimeFormat, so any BCP 47 locale works.
 */
export function relativeTimeLabel(
  date: Date | number,
  now: Date | number = Date.now(),
  locale = "en",
): string {
  const diffSeconds = (new Date(date).getTime() - new Date(now).getTime()) / 1000;
  if (Number.isNaN(diffSeconds)) throw new RangeError("Invalid date");
  const formatter = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
  const absolute = Math.abs(diffSeconds);
  if (absolute < 45) return formatter.format(0, "second");
  for (const [unit, seconds] of UNITS) {
    if (absolute >= seconds || unit === "minute") {
      return formatter.format(Math.round(diffSeconds / seconds), unit);
    }
  }
  return formatter.format(0, "second");
}
