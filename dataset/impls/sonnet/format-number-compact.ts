const UNITS: ReadonlyArray<readonly [number, string]> = [
  [1e3, "K"],
  [1e6, "M"],
  [1e9, "B"],
  [1e12, "T"],
];

function roundHalfAway(value: number, decimals: number): number {
  const scale = 10 ** decimals;
  return Math.round((value + Number.EPSILON * Math.sign(value)) * scale) / scale;
}

function trim(value: number, decimals: number, separator: string): string {
  const text = value.toFixed(decimals);
  const trimmed = text.includes(".") ? text.replace(/\.?0+$/, "") : text;
  return trimmed.replace(".", separator);
}

export function formatCompact(
  value: number,
  opts: { decimals?: number; locale?: "en" | "de-CH" } = {},
): string {
  if (!Number.isFinite(value)) {
    throw new RangeError("value must be a finite number");
  }
  const { decimals = 1, locale = "en" } = opts;
  const separator = locale === "de-CH" ? "," : ".";
  const abs = Math.abs(value);
  let index = UNITS.findLastIndex(([threshold]) => threshold <= abs);
  let scaled = roundHalfAway(index < 0 ? abs : abs / UNITS[index][0], decimals);
  if (index < UNITS.length - 1 && scaled >= 1000) {
    index += 1;
    scaled = roundHalfAway(abs / UNITS[index][0], decimals);
  }
  const suffix = index < 0 ? "" : UNITS[index][1];
  const text = trim(scaled, decimals, separator);
  return value < 0 && scaled !== 0 ? `-${text}${suffix}` : `${text}${suffix}`;
}
