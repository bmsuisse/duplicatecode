export type Condition =
  | { column: string; op: "=" | "!=" | "<" | "<=" | ">" | ">=" | "LIKE" | "ILIKE"; value: unknown }
  | { column: string; op: "IN" | "NOT IN"; value: readonly unknown[] }
  | { column: string; op: "BETWEEN"; value: readonly [unknown, unknown] }
  | { column: string; op: "IS NULL" | "IS NOT NULL" };

export interface WhereClause {
  /** Empty string when there are no conditions, otherwise starts with "WHERE ". */
  sql: string;
  values: unknown[];
}

const IDENTIFIER = /^[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)?$/;

/**
 * Build a parameterised WHERE clause. Column names are validated; values are only
 * returned in `values`. `placeholder` receives the 1-based index ($1) or can be "?".
 */
export function buildWhere(
  conditions: readonly Condition[],
  options: { joiner?: "AND" | "OR"; placeholder?: "?" | "$n" } = {},
): WhereClause {
  const { joiner = "AND", placeholder = "$n" } = options;
  const values: unknown[] = [];
  const bind = (value: unknown): string => {
    values.push(value);
    return placeholder === "?" ? "?" : `$${values.length}`;
  };

  const clauses = conditions.map((cond): string => {
    if (!IDENTIFIER.test(cond.column)) {
      throw new Error(`Invalid column name: ${cond.column}`);
    }
    switch (cond.op) {
      case "IS NULL":
      case "IS NOT NULL":
        return `${cond.column} ${cond.op}`;
      case "IN":
      case "NOT IN":
        if (cond.value.length === 0) return cond.op === "IN" ? "1 = 0" : "1 = 1";
        return `${cond.column} ${cond.op} (${cond.value.map(bind).join(", ")})`;
      case "BETWEEN":
        return `${cond.column} BETWEEN ${bind(cond.value[0])} AND ${bind(cond.value[1])}`;
      default:
        return `${cond.column} ${cond.op} ${bind(cond.value)}`;
    }
  });

  return { sql: clauses.length ? `WHERE ${clauses.join(` ${joiner} `)}` : "", values };
}
