export type Operator =
  | "="
  | "!="
  | "<"
  | "<="
  | ">"
  | ">="
  | "like"
  | "in"
  | "not in"
  | "between"
  | "is null"
  | "is not null";

export interface Condition {
  column: string;
  operator: Operator;
  value?: unknown;
}

export interface ConditionGroup {
  logic: "and" | "or";
  conditions: Array<Condition | ConditionGroup>;
}

export interface WhereClause {
  sql: string;
  params: unknown[];
}

const IDENTIFIER = /^[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)?$/;
const SIMPLE = new Set<string>(["=", "!=", "<", "<=", ">", ">=", "like"]);

function isGroup(node: Condition | ConditionGroup): node is ConditionGroup {
  return "conditions" in node;
}

/**
 * Build a parameterised WHERE clause (without the WHERE keyword). Values are never
 * written into the SQL text, only `?` placeholders. Column names are validated.
 */
export function buildWhereClause(node: Condition | ConditionGroup): WhereClause {
  const params: unknown[] = [];
  const sql = render(node, params);
  return { sql, params };
}

function render(node: Condition | ConditionGroup, params: unknown[]): string {
  if (isGroup(node)) {
    const parts = node.conditions.map((child) => render(child, params)).filter(Boolean);
    if (parts.length === 0) return "";
    if (parts.length === 1) return parts[0] as string;
    return `(${parts.join(node.logic === "or" ? " OR " : " AND ")})`;
  }
  if (!IDENTIFIER.test(node.column)) {
    throw new Error(`Invalid column name: ${node.column}`);
  }
  const column = node.column;
  const op = node.operator;
  switch (op) {
    case "is null":
      return `${column} IS NULL`;
    case "is not null":
      return `${column} IS NOT NULL`;
    case "in":
    case "not in": {
      const values = node.value;
      if (!Array.isArray(values)) throw new Error(`${op} requires an array value`);
      if (values.length === 0) return op === "in" ? "1 = 0" : "1 = 1";
      params.push(...values);
      return `${column} ${op.toUpperCase()} (${values.map(() => "?").join(", ")})`;
    }
    case "between": {
      const range = node.value;
      if (!Array.isArray(range) || range.length !== 2) {
        throw new Error("between requires a [low, high] value");
      }
      params.push(range[0], range[1]);
      return `${column} BETWEEN ? AND ?`;
    }
    default:
      if (!SIMPLE.has(op)) throw new Error(`Unsupported operator: ${String(op)}`);
      params.push(node.value);
      return `${column} ${op.toUpperCase()} ?`;
  }
}
