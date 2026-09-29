export function relativeTimeLabel(date: Date): string {
  const now = new Date();
  const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

  if (seconds < 0) {
    const absSeconds = Math.abs(seconds);
    if (absSeconds < 60) return "in a few seconds";
    if (absSeconds < 3600) return `in ${Math.floor(absSeconds / 60)} minutes`;
    if (absSeconds < 86400) return `in ${Math.floor(absSeconds / 3600)} hours`;
    if (absSeconds < 604800) return `in ${Math.floor(absSeconds / 86400)} days`;
    return "in the future";
  }

  if (seconds < 60) return "just now";
  if (seconds < 3600) return `${Math.floor(seconds / 60)} minutes ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)} hours ago`;
  if (seconds < 604800) return `${Math.floor(seconds / 86400)} days ago`;
  return "a long time ago";
}
