const STEPS: readonly { unit: Intl.RelativeTimeFormatUnit; ms: number }[] = [
  { unit: "year", ms: 365 * 86_400_000 },
  { unit: "month", ms: 30 * 86_400_000 },
  { unit: "week", ms: 7 * 86_400_000 },
  { unit: "day", ms: 86_400_000 },
  { unit: "hour", ms: 3_600_000 },
  { unit: "minute", ms: 60_000 },
];

/** Label like "5 minutes ago", "in 3 days" or "just now" (under a minute). */
export function relativeTimeLabel(
  timestamp: Date | number | string,
  now: Date | number = Date.now(),
  locale = "en",
): string {
  const target = new Date(timestamp).getTime();
  if (Number.isNaN(target)) throw new RangeError("Invalid timestamp");
  const diff = target - new Date(now).getTime();
  const formatter = new Intl.RelativeTimeFormat(locale, { numeric: "always" });

  for (const { unit, ms } of STEPS) {
    if (Math.abs(diff) >= ms) return formatter.format(Math.trunc(diff / ms), unit);
  }
  return "just now";
}
