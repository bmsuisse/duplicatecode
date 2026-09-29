interface StatusBadgeProps {
  status: "active" | "pending" | "error" | "inactive";
  label?: string;
}

export function StatusBadge({ status, label }: StatusBadgeProps) {
  const statusColors = {
    active: "bg-green-100 text-green-800",
    pending: "bg-yellow-100 text-yellow-800",
    error: "bg-red-100 text-red-800",
    inactive: "bg-gray-100 text-gray-800",
  };

  const statusLabels = {
    active: "Active",
    pending: "Pending",
    error: "Error",
    inactive: "Inactive",
  };

  return (
    <span className={`px-2.5 py-0.5 rounded-full text-sm font-medium ${statusColors[status]}`}>
      {label || statusLabels[status]}
    </span>
  );
}
