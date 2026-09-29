export function formatCompact(num: number): string {
  const abs = Math.abs(num);
  const sign = num < 0 ? "-" : "";

  if (abs >= 1_000_000) {
    return `${sign + (abs / 1_000_000).toFixed(1)}M`;
  } else if (abs >= 1_000) {
    return `${sign + (abs / 1_000).toFixed(1)}K`;
  } else {
    return sign + abs.toString();
  }
}
