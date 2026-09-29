export interface WhereClause {
  clause: string;
  params: any[];
}

export function buildWhereClause(filters: Record<string, any>): WhereClause {
  const conditions: string[] = [];
  const params: any[] = [];

  for (const [key, value] of Object.entries(filters)) {
    conditions.push(`${key} = $${params.length + 1}`);
    params.push(value);
  }

  const clause = conditions.length > 0 ? conditions.join(" AND ") : "1=1";
  return { clause, params };
}
