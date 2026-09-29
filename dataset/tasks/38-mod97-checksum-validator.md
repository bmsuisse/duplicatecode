# Mod-97 account number validation

- **Language:** typescript
- **Target file:** `mod97-checksum-validator.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export function isValidIban(value: string): boolean
export function formatIban(value: string): string
```

## Behavior

isValidIban: strip spaces and hyphens, uppercase; must match /^[A-Z]{2}[0-9]{2}[A-Z0-9]{11,30}$/; move first four characters to the end; map letters A=10..Z=35; the integer mod 97 must be 1. Must not use BigInt or Number for the whole digit string (compute piecewise). No per-country length checks. formatIban: normalised value grouped in blocks of 4 separated by a single space; throws Error if invalid.

## Edge cases

- Empty / non-matching -> false.
- Lowercase and spaces accepted.

## Examples

- 'DE89 3704 0044 0532 0130 00' -> true
- 'gb82west12345698765432' -> true
- 'DE89370400440532013001' -> false
- '12345' -> false
- formatIban('ch9300762011623852957') -> 'CH93 0076 2011 6238 5295 7'
