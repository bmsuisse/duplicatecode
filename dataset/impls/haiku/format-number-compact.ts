export function formatCompact(
  value: number,
  opts?: { decimals?: number; locale?: "en" | "de-CH" },
): string {
  if (!Number.isFinite(value)) {
    throw new RangeError("value must be finite");
  }

  const decimals = opts?.decimals ?? 1;
  const locale = opts?.locale ?? "en";
  const separator = locale === "de-CH" ? "," : ".";

  const units = [
    { threshold: 1e12, suffix: "T" },
    { threshold: 1e9, suffix: "B" },
    { threshold: 1e6, suffix: "M" },
    { threshold: 1e3, suffix: "K" },
  ];

  const absValue = Math.abs(value);

  // Find the appropriate unit
  let unit = units.find((u) => absValue >= u.threshold);

  if (!unit) {
    // No suffix
    return value.toString();
  }

  // Divide by the threshold
  let divided = absValue / unit.threshold;

  // Round half away from zero
  const factor = 10 ** decimals;
  divided = (Math.sign(divided) * Math.round(Math.abs(divided) * factor)) / factor;

  // Check if rounding promoted to next suffix
  if (divided >= 1000 && unit !== units[units.length - 1]) {
    const nextUnitIdx = units.indexOf(unit) + 1;
    if (nextUnitIdx < units.length) {
      unit = units[nextUnitIdx];
      divided = absValue / unit.threshold;
      divided = (Math.sign(divided) * Math.round(Math.abs(divided) * factor)) / factor;
    }
  }

  // Format the number
  let formatted = divided.toFixed(decimals);

  // Trim trailing zeros and dangling decimal point
  formatted = formatted.replace(/\.?0+$/, "");

  // Add sign if negative
  if (value < 0) {
    formatted = `${"-"}${formatted}`;
  }

  // Replace decimal separator if needed
  if (separator === ",") {
    formatted = formatted.replace(".", ",");
  }

  return formatted + unit.suffix;
}
