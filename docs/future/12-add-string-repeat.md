# Add: String Repeat

> Status: **Done**.

## Problem

Drawing boxes, frames, and separators requires manual loops:

```pascal
mutable var Line: string := '';
for I: integer := 1 to Width do
  Line := Line + '─';
```

## Solution

`RepeatStr` is available in `Std.Str`:

```pascal
var Line: string := RepeatStr('─', Width);
var Border: string := RepeatStr('═', 40);
```

> **Note:** The name `Repeat` cannot be used because `repeat` is a reserved keyword (`repeat … until` loop).

### Signature

```
function RepeatStr(S: string; Count: integer): string;
```

- Returns `S` concatenated `Count` times.
- `Count <= 0` returns an empty string.

## Scope

- **fpas-std:** implemented in `Std.Str`.
- **Std registry:** registered as `RepeatStr` in `Std.Str`.
- **Docs:** documented in `std/str.md`.

No language changes. No new keywords.
