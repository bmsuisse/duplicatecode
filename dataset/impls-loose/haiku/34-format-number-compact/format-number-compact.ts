export function formatCompact(num: number): string {
  const abs = Math.abs(num);

  if (abs >= 1e9) {
    return (num / 1e9).toFixed(1).replace(/\.0+$/, "") + "B";
  }
  if (abs >= 1e6) {
    return (num / 1e6).toFixed(1).replace(/\.0+$/, "") + "M";
  }
  if (abs >= 1e3) {
    return (num / 1e3).toFixed(1).replace(/\.0+$/, "") + "K";
  }

  return num.toString();
}
