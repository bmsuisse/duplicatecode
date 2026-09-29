# Compact number formatting

- **Language:** typescript
- **Target file:** `format-number-compact.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export function formatCompact(value: number, opts?: { decimals?: number; locale?: 'en' | 'de-CH' }): string
```

## Behavior

Formats magnitudes with suffix K (1e3), M (1e6), B (1e9), T (1e12). Choose the largest suffix whose threshold <= |value|; below 1000 no suffix. Divide by the unit, round half away from zero to `decimals` (default 1), trim trailing zeros and any dangling decimal point, append suffix. If rounding reaches 1000 in the chosen unit, promote to the next suffix (999950 -> '1M'). Decimal separator: '.' for locale 'en' (default), ',' for 'de-CH'. Negative values keep a leading '-'. NaN/Infinity -> throws RangeError.

## Edge cases

- 0 -> '0'.
- -1500 -> '-1.5K'.
- decimals 0 supported.

## Examples

- 999 -> '999'
- 1500 -> '1.5K'
- 999950 -> '1M'
- 2_345_678_901 , decimals 2 -> '2.35B'
- 1500, {locale:'de-CH'} -> '1,5K'
