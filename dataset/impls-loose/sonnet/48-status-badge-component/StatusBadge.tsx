export type Status = "active" | "pending" | "error" | "inactive" | "success" | "warning";

const STYLES: Record<Status, { background: string; color: string }> = {
  active: { background: "#dcfce7", color: "#166534" },
  success: { background: "#dcfce7", color: "#166534" },
  pending: { background: "#fef9c3", color: "#854d0e" },
  warning: { background: "#ffedd5", color: "#9a3412" },
  error: { background: "#fee2e2", color: "#991b1b" },
  inactive: { background: "#f3f4f6", color: "#374151" },
};

export interface StatusBadgeProps {
  status: Status;
  /** Text to show; defaults to the capitalised status. */
  label?: string;
}

export function StatusBadge({ status, label }: StatusBadgeProps) {
  const { background, color } = STYLES[status] ?? STYLES.inactive;
  const text = label ?? status.charAt(0).toUpperCase() + status.slice(1);
  return (
    <span
      data-status={status}
      style={{
        display: "inline-block",
        padding: "2px 10px",
        borderRadius: 9999,
        fontSize: 12,
        fontWeight: 600,
        lineHeight: "18px",
        background,
        color,
      }}
    >
      {text}
    </span>
  );
}
