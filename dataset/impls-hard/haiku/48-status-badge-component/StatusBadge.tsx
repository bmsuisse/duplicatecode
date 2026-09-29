interface StatusBadgeProps {
  status: "success" | "error" | "warning" | "info";
  label: string;
}

export function StatusBadge({ status, label }: StatusBadgeProps) {
  const colorMap = {
    success: "bg-green-100 text-green-800",
    error: "bg-red-100 text-red-800",
    warning: "bg-yellow-100 text-yellow-800",
    info: "bg-blue-100 text-blue-800",
  };

  return (
    <span className={`px-2 py-1 rounded text-sm font-medium ${colorMap[status]}`}>{label}</span>
  );
}
