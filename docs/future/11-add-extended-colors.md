# Add: Extended Color Support (256-color / Truecolor)

> Priority: 11 — stdlib extension after core language is stable.

## Problem

`Std.Console` only supports 16 CRT colors (0–15). Modern terminals support
256-color and 24-bit truecolor palettes. Richer color support enables smoother
gradients and better visuals.

## Solution

Add to `Std.Console`:

```pascal
procedure TextColorRGB(R: integer; G: integer; B: integer);
procedure TextBackgroundRGB(R: integer; G: integer; B: integer);
procedure TextColor256(Index: integer);
procedure TextBackground256(Index: integer);
```

- `TextColor256` / `TextBackground256`: 256-color palette index (0–255).
- `TextColorRGB` / `TextBackgroundRGB`: 24-bit truecolor (0–255 per channel).
- Runtime error for out-of-range values.

## Scope

- **fpas-std:** implement extended color output via ANSI escape sequences
  (the Rust `crossterm` crate already supports this).
- **Std registry:** register 4 new procedures in `Std.Console`.
- **Compiler:** standard call compilation, no special handling needed.
- **Docs:** add to `std/console.md`.

No language changes. No new keywords.
