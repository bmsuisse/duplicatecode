# Parameterised SQL WHERE builder

- **Language:** typescript
- **Target file:** `where-clause-builder.ts`
- **Constraints:** No npm packages other than `react` (types for react are NOT installed, so avoid importing type names from react; the file must lint cleanly with biome and use only explicit local types). Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export type Filter = { field: string; op: 'eq'|'ne'|'gt'|'gte'|'lt'|'lte'|'like'|'in'|'notIn'|'isNull'|'between'; value?: unknown }
export function buildWhere(filters: readonly Filter[], opts?: { placeholder?: '?' | '$n' }): { sql: string; params: unknown[] }
```

## Behavior

Join filters with AND into 'WHERE ...'. Operators: eq '=', ne '<>', gt '>', gte '>=', lt '<', lte '<=', like 'LIKE'; in / notIn -> 'IN (..)' / 'NOT IN (..)'; isNull -> value true: 'IS NULL', false: 'IS NOT NULL'; between -> 'BETWEEN p AND p' with a 2-element array. eq with null -> 'IS NULL' (no param); ne with null -> 'IS NOT NULL'. `in` with empty array -> '1 = 0'; `notIn` with empty array is skipped. Placeholder '?' (default) or '$n' numbered from $1 in param order. Field must match /^[A-Za-z_][A-Za-z0-9_.]*$/ else throw Error. Empty filters -> {sql:'', params:[]}.

## Edge cases

- Unknown op or malformed between -> throws.
- Params ordered by appearance; skipped filters add no params.

## Examples

- [{field:'age',op:'gte',value:18},{field:'city',op:'in',value:['A','B']}] -> sql 'WHERE age >= ? AND city IN (?, ?)', params [18,'A','B']
- same with '$n' -> 'WHERE age >= $1 AND city IN ($2, $3)'
- [{field:'x',op:'eq',value:null}] -> 'WHERE x IS NULL', []
- [{field:'n',op:'between',value:[1,5]}] -> 'WHERE n BETWEEN ? AND ?'
- [] -> {sql:'',params:[]}
