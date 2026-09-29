# Relative time label

- **Language:** typescript
- **Target file:** `relative-time-label.ts`
- **Constraints:** No npm packages at all; pure TypeScript, strict mode. Single self-contained file, 20-80 lines of implementation, no I/O beyond what is described.

## Signature

```ts
export function relativeTime(target: Date | number, now: Date | number, opts?: { locale?: 'en' | 'de' }): string
```

## Behavior

Describe the difference between target and now (d = absolute difference in whole seconds, floored). d < 60 -> 'just now' (de 'gerade eben'), no suffix. d < 3600 -> floor(d/60) minutes; d < 86400 -> floor(d/3600) hours; d < 30 days -> floor(d/86400) days; d < 365 days -> floor(d/(30*86400)) months; else floor(d/(365*86400)) years. Singular unit when N == 1 ('1 minute'). English: past 'N units ago', future 'in N units'. German: past 'vor N <unit>', future 'in N <unit>', with units Minute/Minuten, Stunde/Stunden, Tag/Tagen, Monat/Monaten, Jahr/Jahren (singular for N == 1; plural forms are dative as shown, e.g. 'vor 3 Tagen', 'vor 1 Tag').

## Edge cases

- Exactly 60 s -> '1 minute ago'.
- Accepts Date or epoch ms.

## Examples

- now-30s -> 'just now'
- now-5min -> '5 minutes ago'
- now+2h -> 'in 2 hours'
- now-1 day -> '1 day ago'
- now-3 days, locale de -> 'vor 3 Tagen'
