export type Filter = {
  field: string;
  op: "eq" | "ne" | "gt" | "gte" | "lt" | "lte" | "like" | "in" | "notIn" | "isNull" | "between";
  value?: unknown;
};

const FIELD_PATTERN = /^[A-Za-z_][A-Za-z0-9_.]*$/;
const COMPARISONS: Record<string, string> = {
  eq: "=",
  ne: "<>",
  gt: ">",
  gte: ">=",
  lt: "<",
  lte: "<=",
  like: "LIKE",
};

export function buildWhere(
  filters: readonly Filter[],
  opts: { placeholder?: "?" | "$n" } = {},
): { sql: string; params: unknown[] } {
  const params: unknown[] = [];
  const clauses: string[] = [];
  const bind = (value: unknown): string => {
    params.push(value);
    return opts.placeholder === "$n" ? `$${params.length}` : "?";
  };

  for (const { field, op, value } of filters) {
    if (!FIELD_PATTERN.test(field)) {
      throw new Error(`Invalid field name: ${field}`);
    }
    if ((op === "eq" || op === "ne") && value === null) {
      clauses.push(`${field} IS ${op === "ne" ? "NOT " : ""}NULL`);
    } else if (op in COMPARISONS) {
      clauses.push(`${field} ${COMPARISONS[op]} ${bind(value)}`);
    } else if (op === "in" || op === "notIn") {
      const list = Array.isArray(value) ? value : [];
      if (list.length === 0) {
        if (op === "in") {
          clauses.push("1 = 0");
        }
        continue;
      }
      clauses.push(`${field} ${op === "in" ? "IN" : "NOT IN"} (${list.map(bind).join(", ")})`);
    } else if (op === "isNull") {
      clauses.push(`${field} IS ${value ? "NULL" : "NOT NULL"}`);
    } else if (op === "between") {
      if (!Array.isArray(value) || value.length !== 2) {
        throw new Error("between requires a two-element array");
      }
      clauses.push(`${field} BETWEEN ${bind(value[0])} AND ${bind(value[1])}`);
    } else {
      throw new Error(`Unknown operator: ${op}`);
    }
  }
  return { sql: clauses.length ? `WHERE ${clauses.join(" AND ")}` : "", params };
}
