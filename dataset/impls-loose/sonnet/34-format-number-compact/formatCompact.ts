const UNITS = [
  { value: 1e12, suffix: "T" },
  { value: 1e9, suffix: "B" },
  { value: 1e6, suffix: "M" },
  { value: 1e3, suffix: "K" },
] as const;

/** Format 1234 as "1.2K", 3_400_000 as "3.4M". Trailing ".0" is dropped. */
export function formatCompact(value: number, fractionDigits = 1): string {
  if (!Number.isFinite(value)) return String(value);
  const sign = value < 0 ? "-" : "";
  const abs = Math.abs(value);
  const trim = (n: number): string => n.toFixed(fractionDigits).replace(/\.?0+$/, "");

  for (let i = 0; i < UNITS.length; i++) {
    const unit = UNITS[i];
    if (abs < unit.value) continue;
    const scaled = Number(trim(abs / unit.value));
    // 999_999 rounds to 1000K; promote it to 1M.
    const larger = UNITS[i - 1];
    if (scaled >= 1000 && larger) return `${sign}${trim(abs / larger.value)}${larger.suffix}`;
    return `${sign}${trim(abs / unit.value)}${unit.suffix}`;
  }
  return `${sign}${trim(abs)}`;
}
