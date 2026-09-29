export function relativeTime(
  target: Date | number,
  now: Date | number,
  opts?: { locale?: "en" | "de" },
): string {
  const locale = opts?.locale ?? "en";

  const targetMs = target instanceof Date ? target.getTime() : target;
  const nowMs = now instanceof Date ? now.getTime() : now;

  const diffMs = Math.abs(targetMs - nowMs);
  const d = Math.floor(diffMs / 1000); // Whole seconds

  const isFuture = targetMs > nowMs;

  if (d < 60) {
    return locale === "de" ? "gerade eben" : "just now";
  }

  let value: number;
  let unit: string;

  if (d < 3600) {
    value = Math.floor(d / 60);
    unit =
      locale === "de" ? (value === 1 ? "Minute" : "Minuten") : value === 1 ? "minute" : "minutes";
  } else if (d < 86400) {
    value = Math.floor(d / 3600);
    unit = locale === "de" ? (value === 1 ? "Stunde" : "Stunden") : value === 1 ? "hour" : "hours";
  } else if (d < 30 * 86400) {
    value = Math.floor(d / 86400);
    unit = locale === "de" ? (value === 1 ? "Tag" : "Tagen") : value === 1 ? "day" : "days";
  } else if (d < 365 * 86400) {
    value = Math.floor(d / (30 * 86400));
    unit = locale === "de" ? (value === 1 ? "Monat" : "Monaten") : value === 1 ? "month" : "months";
  } else {
    value = Math.floor(d / (365 * 86400));
    unit = locale === "de" ? (value === 1 ? "Jahr" : "Jahren") : value === 1 ? "year" : "years";
  }

  if (locale === "de") {
    return isFuture ? `in ${value} ${unit}` : `vor ${value} ${unit}`;
  } else {
    return isFuture ? `in ${value} ${unit}` : `${value} ${unit} ago`;
  }
}
