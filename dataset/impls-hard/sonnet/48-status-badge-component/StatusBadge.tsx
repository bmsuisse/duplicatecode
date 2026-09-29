import type { CSSProperties } from "react";

export type Status = "active" | "pending" | "in-progress" | "error" | "inactive";

const STATUS_STYLES: Record<Status, { label: string; background: string; color: string }> = {
  active: { label: "Active", background: "#dcfce7", color: "#166534" },
  pending: { label: "Pending", background: "#fef9c3", color: "#854d0e" },
  "in-progress": { label: "In progress", background: "#dbeafe", color: "#1e40af" },
  error: { label: "Error", background: "#fee2e2", color: "#991b1b" },
  inactive: { label: "Inactive", background: "#e5e7eb", color: "#374151" },
};

export interface StatusBadgeProps {
  status: Status;
  /** Overrides the default label. */
  label?: string;
  className?: string;
}

/** Small coloured pill that shows the state of an item. */
export function StatusBadge({ status, label, className }: StatusBadgeProps) {
  const style = STATUS_STYLES[status];
  const css: CSSProperties = {
    display: "inline-block",
    padding: "2px 10px",
    borderRadius: 999,
    fontSize: 12,
    fontWeight: 600,
    lineHeight: "18px",
    whiteSpace: "nowrap",
    background: style.background,
    color: style.color,
  };
  return (
    <span className={className} style={css} data-status={status}>
      {label ?? style.label}
    </span>
  );
}
