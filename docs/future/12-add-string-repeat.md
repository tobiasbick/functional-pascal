# Add: String Repeat

> Priority: 12 — small stdlib utility.

## Problem

Drawing boxes, frames, and separators requires manual loops:

```pascal
mutable var Line: string := '';
for I: integer := 1 to Width do
  Line := Line + '─';
```

## Solution

Add `Repeat` to `Std.Str`:

```pascal
var Line: string := Repeat('─', Width);
var Border: string := Repeat('═', 40);
```

### Signature

```
function Repeat(S: string; Count: integer): string;
```

- Returns `S` concatenated `Count` times.
- `Count <= 0` returns an empty string.

## Scope

- **fpas-std:** implement in `Std.Str`.
- **Std registry:** register `Repeat` in `Std.Str`.
- **Docs:** add to `std/str.md`.

No language changes. No new keywords.
