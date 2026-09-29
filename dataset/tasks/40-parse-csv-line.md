# Parse a CSV text into rows

- **Language:** typescript
- **Target file:** `parse-csv-line.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export function parseCsv(text: string, opts?: { delimiter?: string; trim?: boolean }): string[][]
```

## Behavior

RFC-4180-style parser. Delimiter default ','. Fields may be quoted with '"'; inside quotes the delimiter and newlines are literal and '""' is an escaped quote. Rows end on \n, \r\n or \r. Lines that are entirely empty (outside quotes) are skipped, so a trailing newline yields no extra row. With trim:true unquoted fields are trimmed (quoted ones are not). Unterminated quote -> throws Error.

## Edge cases

- Empty text -> [].
- Quoted empty field -> ''.
- Whitespace outside quotes retained unless trim.

## Examples

- 'a,b\n1,2\n' -> [['a','b'],['1','2']]
- 'x,"he said ""hi""",z' -> [['x','he said "hi"','z']]
- '"a,b",c\r\n"line1\nline2",d' -> [['a,b','c'],['line1\nline2','d']]
- 'a;b', {delimiter:';'} -> [['a','b']]
- '"abc' -> throws
