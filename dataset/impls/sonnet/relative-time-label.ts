type Locale = "en" | "de";

const DAY = 86400;
const UNIT_NAMES: Record<Locale, Record<string, [string, string]>> = {
  en: {
    minute: ["minute", "minutes"],
    hour: ["hour", "hours"],
    day: ["day", "days"],
    month: ["month", "months"],
    year: ["year", "years"],
  },
  de: {
    minute: ["Minute", "Minuten"],
    hour: ["Stunde", "Stunden"],
    day: ["Tag", "Tagen"],
    month: ["Monat", "Monaten"],
    year: ["Jahr", "Jahren"],
  },
};

export function relativeTime(
  target: Date | number,
  now: Date | number,
  opts: { locale?: Locale } = {},
): string {
  const locale = opts.locale ?? "en";
  const diffMs =
    Number(target instanceof Date ? target.getTime() : target) -
    Number(now instanceof Date ? now.getTime() : now);
  const seconds = Math.floor(Math.abs(diffMs) / 1000);
  if (seconds < 60) {
    return locale === "de" ? "gerade eben" : "just now";
  }

  let unit: string;
  let count: number;
  if (seconds < 3600) {
    [unit, count] = ["minute", Math.floor(seconds / 60)];
  } else if (seconds < DAY) {
    [unit, count] = ["hour", Math.floor(seconds / 3600)];
  } else if (seconds < 30 * DAY) {
    [unit, count] = ["day", Math.floor(seconds / DAY)];
  } else if (seconds < 365 * DAY) {
    [unit, count] = ["month", Math.floor(seconds / (30 * DAY))];
  } else {
    [unit, count] = ["year", Math.floor(seconds / (365 * DAY))];
  }

  const [singular, plural] = UNIT_NAMES[locale][unit];
  const phrase = `${count} ${count === 1 ? singular : plural}`;
  const future = diffMs > 0;
  if (locale === "de") {
    return future ? `in ${phrase}` : `vor ${phrase}`;
  }
  return future ? `in ${phrase}` : `${phrase} ago`;
}
