# Add: String Formatting

> Priority: 10 — high-value stdlib addition after removals are done.

## Problem

Building display strings is painful:

```pascal
var Status: string := 'Zoom: ' + RealToStr(Zoom) + 'x Center: ('
  + RealToStr(CX) + ', ' + RealToStr(CY) + ')';
```

TUI apps need status bars, coordinate displays, and parameter readouts.
Every one requires manual type conversion and string concatenation.

## Solution

Add a `Format` function to `Std.Str`:

```pascal
var Status: string := Format('Zoom: %fx Center: (%f, %f)', Zoom, CX, CY);
```

### Proposed format specifiers

| Specifier | Type | Example |
|-----------|------|---------|
| `%d` | `integer` | `Format('%d', 42)` → `'42'` |
| `%f` | `real` | `Format('%f', 3.14)` → `'3.14'` |
| `%s` | `string` | `Format('%s', 'hi')` → `'hi'` |
| `%%` | literal `%` | `Format('100%%')` → `'100%'` |

Variadic — accepts any number of arguments matching the specifiers.
Mismatch between specifiers and arguments is a runtime error.

## Scope

- **fpas-std:** implement `format` function in `Std.Str`.
- **Std registry:** register `Format` in `Std.Str` unit.
- **Compiler:** handle variadic call for `Format` (same mechanism as
  `Write`/`WriteLn`).
- **Docs:** add to `std/str.md`.

No language changes. No new keywords.
