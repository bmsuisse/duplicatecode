export function buildWhereClause(filters: Record<string, any>): [string, any[]] {
  const conditions: string[] = [];
  const values: any[] = [];

  for (const [key, value] of Object.entries(filters)) {
    if (value === null || value === undefined) {
      conditions.push(`${key} IS NULL`);
    } else {
      conditions.push(`${key} = ?`);
      values.push(value);
    }
  }

  const whereClause = conditions.length > 0 ? conditions.join(" AND ") : "1=1";
  return [whereClause, values];
}
