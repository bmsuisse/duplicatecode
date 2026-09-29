export type Status = "active" | "pending" | "inactive" | "error";
export interface StatusBadgeProps {
  status: Status;
  label?: string;
  size?: "sm" | "md";
}

const DEFAULT_LABELS: Record<Status, string> = {
  active: "Active",
  pending: "Pending",
  inactive: "Inactive",
  error: "Error",
};

export function StatusBadge({ status, label, size = "md" }: StatusBadgeProps) {
  const known = Object.hasOwn(DEFAULT_LABELS, status);
  const modifier = known ? status : "unknown";
  const text = label || (known ? DEFAULT_LABELS[status] : String(status));
  return (
    <span
      className={`badge badge--${modifier} badge--${known ? size : "md"}`}
      role="status"
      data-status={status}
    >
      {text}
    </span>
  );
}
