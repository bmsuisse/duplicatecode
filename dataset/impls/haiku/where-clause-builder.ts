export type Filter = {
  field: string;
  op: "eq" | "ne" | "gt" | "gte" | "lt" | "lte" | "like" | "in" | "notIn" | "isNull" | "between";
  value?: unknown;
};

export function buildWhere(
  filters: readonly Filter[],
  opts?: { placeholder?: "?" | "$n" },
): { sql: string; params: unknown[] } {
  const placeholder = opts?.placeholder ?? "?";

  if (filters.length === 0) {
    return { sql: "", params: [] };
  }

  const opMap: Record<string, string> = {
    eq: "=",
    ne: "<>",
    gt: ">",
    gte: ">=",
    lt: "<",
    lte: "<=",
    like: "LIKE",
  };

  const clauses: string[] = [];
  const params: unknown[] = [];

  for (const filter of filters) {
    const { field, op, value } = filter;

    // Validate field
    if (!/^[A-Za-z_][A-Za-z0-9_.]*$/.test(field)) {
      throw new Error(`Invalid field name: ${field}`);
    }

    if (op === "eq") {
      if (value === null) {
        clauses.push(`${field} IS NULL`);
      } else {
        const paramPlaceholder = placeholder === "?" ? "?" : `$${params.length + 1}`;
        clauses.push(`${field} = ${paramPlaceholder}`);
        params.push(value);
      }
    } else if (op === "ne") {
      if (value === null) {
        clauses.push(`${field} IS NOT NULL`);
      } else {
        const paramPlaceholder = placeholder === "?" ? "?" : `$${params.length + 1}`;
        clauses.push(`${field} <> ${paramPlaceholder}`);
        params.push(value);
      }
    } else if (op === "isNull") {
      const isNull = value === true;
      clauses.push(`${field} IS ${isNull ? "NULL" : "NOT NULL"}`);
    } else if (op === "in") {
      if (!Array.isArray(value)) {
        throw new Error("in operator requires an array value");
      }
      if (value.length === 0) {
        clauses.push("1 = 0");
      } else {
        const placeholders = value
          .map((_, i) => (placeholder === "?" ? "?" : `$${params.length + i + 1}`))
          .join(", ");
        clauses.push(`${field} IN (${placeholders})`);
        params.push(...value);
      }
    } else if (op === "notIn") {
      if (!Array.isArray(value)) {
        throw new Error("notIn operator requires an array value");
      }
      if (value.length === 0) {
        // Skip
      } else {
        const placeholders = value
          .map((_, i) => (placeholder === "?" ? "?" : `$${params.length + i + 1}`))
          .join(", ");
        clauses.push(`${field} NOT IN (${placeholders})`);
        params.push(...value);
      }
    } else if (op === "between") {
      if (!Array.isArray(value) || value.length !== 2) {
        throw new Error("between operator requires a 2-element array");
      }
      const p1 = placeholder === "?" ? "?" : `$${params.length + 1}`;
      const p2 = placeholder === "?" ? "?" : `$${params.length + 2}`;
      clauses.push(`${field} BETWEEN ${p1} AND ${p2}`);
      params.push(value[0], value[1]);
    } else if (op in opMap) {
      const paramPlaceholder = placeholder === "?" ? "?" : `$${params.length + 1}`;
      clauses.push(`${field} ${opMap[op]} ${paramPlaceholder}`);
      params.push(value);
    } else {
      throw new Error(`Unknown operator: ${op}`);
    }
  }

  const sql = clauses.length > 0 ? `WHERE ${clauses.join(" AND ")}` : "";

  return { sql, params };
}
