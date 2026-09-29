export type Status = "active" | "pending" | "inactive" | "error";

export interface StatusBadgeProps {
  status: Status;
  label?: string;
  size?: "sm" | "md";
}

export function StatusBadge(props: StatusBadgeProps) {
  const { status, label, size = "md" } = props;

  const defaultTexts: Record<Status, string> = {
    active: "Active",
    pending: "Pending",
    inactive: "Inactive",
    error: "Error",
  };

  const isValidStatus = status in defaultTexts;
  const displayStatus = isValidStatus ? status : "unknown";
  const text =
    label && label.trim() !== "" ? label : defaultTexts[status as Status] || (status as string);
  const className = `badge badge--${displayStatus} badge--${size}`;

  return (
    <span className={className} role="status" data-status={status}>
      {text}
    </span>
  );
}
