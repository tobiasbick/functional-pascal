# Cells and frames

The cell API paints a retained text screen using explicit glyph and color values. It is intended for
fullscreen programs that need predictable updates, bulk row drawing, screen read-back, or temporary
overlays.

All coordinates in this API are **1-based and screen-absolute**. They do not use the active
[`Window`](screen.md) or its relative cursor.

## Quick reference

```pascal
type
  ColorKind = enum
    Crt;
    Ansi256;
    Rgb;
  end;
  Color = record
    kind: ColorKind;
    index: integer;
    red: integer;
    green: integer;
    blue: integer;
  end;
  Cell = record
    glyph: string;
    foreground: Color;
    background: Color;
  end;
  Rect = record
    x: integer;
    y: integer;
    width: integer;
    height: integer;
  end;
  SavedRegion = record
  end;
```

| Symbol | Result | Purpose |
|--------|--------|---------|
| `CrtColor(Index)` | `Color` | Construct a classic 16-color value (`0..15`). |
| `Ansi256Color(Index)` | `Color` | Construct an ANSI palette value (`0..255`). |
| `RgbColor(Red, Green, Blue)` | `Color` | Construct a 24-bit color (`0..255` per channel). |
| `BeginFrame()` | — | Begin or nest a deferred frame. |
| `Present()` | — | End one frame level and flush the outermost frame. |
| `PutCell(X, Y, Value)` | — | Paint one logical glyph. |
| `GetCell(X, Y)` | `Option of Cell` | Read one logical glyph. |
| `FillRect(Bounds, Value)` | — | Fill a clipped rectangle with a one-column glyph. |
| `WriteCells(X, Y, Values)` | — | Paint an array of cells from left to right. |
| `SaveRegion(Bounds)` | `SavedRegion` | Capture a clipped region in a one-shot handle. |
| `RestoreRegion(Region)` | — | Restore and consume a saved region. |
| `DiscardRegion(Region)` | — | Consume a saved region without restoring it. |
| `DisplayWidth(Text)` | `integer` | Return the terminal-column width of Unicode text by extended grapheme cluster. |

## Colors and cells

Use the constructors instead of assembling a `Color` record by hand:

```pascal
var Accent: Color := RgbColor(255, 160, 32);
var Tile: Cell := record
  glyph := 'A';
  foreground := Accent;
  background := CrtColor(Black);
end;
```

`CrtColor` accepts `0..15`, including the named CRT constants such as `Black`, `LightGray`, and
`White`. `Ansi256Color` and each channel of `RgbColor` accept `0..255`. Values outside those ranges
produce a runtime error.

All three color modes are stored in each cell and round-trip through `PutCell` / `GetCell`,
`WriteCells`, and saved regions. The fields not used by a color's `kind` are representation details;
construct colors with the functions above and inspect `kind` plus the matching fields when needed:

- `ColorKind.Crt` and `ColorKind.Ansi256` use `index`.
- `ColorKind.Rgb` uses `red`, `green`, and `blue`.

`DisplayWidth` segments text into extended grapheme clusters, so a base glyph with combining marks
or a joined emoji sequence is measured as one renderable unit. Each cluster occupies zero, one, or
two terminal columns; ambiguous-width characters use one column.

`Cell.glyph` must contain one Unicode scalar with non-zero display width. A standalone combining
mark or another zero-width glyph is rejected. Use `DisplayWidth` when laying out strings whose
terminal width may differ from their character count.

## Drawing

`PutCell` paints at an absolute screen coordinate. Coordinates outside the screen are ignored.

`FillRect` clips `Bounds` to the screen. Its glyph must occupy exactly one terminal column because
the same cell is repeated at every coordinate. Rectangle coordinates and dimensions must be
positive; a fully off-screen rectangle has no visible effect.

`WriteCells` starts at `(X, Y)` and advances by each glyph's terminal display width. Painting stops
at the screen edge. A wide glyph reserves its following column as a continuation:

```pascal
var Cells: array of Cell := [
  record glyph := 'A'; foreground := CrtColor(White); background := CrtColor(Black); end,
  record glyph := '中'; foreground := RgbColor(80, 200, 255); background := CrtColor(Black); end
];
WriteCells(1, 1, Cells)
```

`GetCell` returns:

- `Some(Cell)` at the leading column of a painted glyph;
- `None` outside the screen;
- `None` at a continuation column reserved by a wide glyph.

Overwriting either part of a wide glyph repairs the affected columns so no stale continuation
remains.

## Frames

Without a frame, cell operations and existing console operations remain immediate. A frame batches
screen changes until `Present`, reducing visible tearing in fullscreen redraws:

```pascal
BeginFrame();
FillRect(
  record x := 1; y := 1; width := ScreenWidth(); height := ScreenHeight(); end,
  record glyph := ' '; foreground := CrtColor(LightGray); background := CrtColor(Black); end
);
WriteCells(1, 1, Cells);
Present()
```

Frames may nest. Each `BeginFrame` requires a matching `Present`; an inner `Present` only decreases
the nesting depth, and only the outermost call flushes. Calling `Present` when no frame is active
explicitly flushes the pending screen state.

Legacy cursor and text operations can be used during a frame. Their state changes are presented
with the cell updates; outside a frame their existing immediate behavior is unchanged.

## Saved regions

`SaveRegion` captures the clipped cells in `Bounds` and returns an opaque `SavedRegion`. The
rectangle must overlap the screen and have positive dimensions.

Each handle is one-shot:

```pascal
var Underlay: SavedRegion :=
  SaveRegion(record x := 10; y := 4; width := 24; height := 5; end);
FillRect(
  record x := 10; y := 4; width := 24; height := 5; end,
  record glyph := ' '; foreground := CrtColor(White); background := Ansi256Color(24); end
);
RestoreRegion(Underlay)
```

`RestoreRegion` restores the captured cells and consumes the handle. `DiscardRegion` consumes it
without changing the screen. Reusing an already restored or discarded handle is a runtime error.
Restoring after a resize that made the captured rectangle unavailable is also an error. Restore
inside a frame when the overlay removal should be part of the next atomic presentation.

## Fullscreen redraw pattern

For an interactive application:

1. Enable raw mode and the alternate screen, then hide the cursor.
2. On redraw, call `BeginFrame`, use `FillRect` and row-oriented `WriteCells`, then call `Present`.
3. Wait for structured events and redraw only when state changes.
4. Restore terminal modes and cursor visibility during shutdown.

See the multi-unit [Mandelbrot example](../../../../examples/math/mandelbrot/README.md) for this
pattern with concurrent row calculation and RGB cell colors.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Public type and call registration | [`loaded/console.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/console.rs) |
| Compiler lowering | [`std_calls/console/`](../../../../crates/fpas-compiler/src/compiler/std_calls/console/mod.rs) |
| Cell/color values | [`console/cell.rs`](../../../../crates/fpas-std/src/console/cell.rs) |
| Cell, frame, and region operations | [`console/operations/`](../../../../crates/fpas-std/src/console/operations/mod.rs) |
| Retained screen storage | [`console/screen/`](../../../../crates/fpas-std/src/console/screen/mod.rs) |

## See also

- [Console overview](README.md)
- [Colors and attributes](colors.md)
- [Screen control](screen.md)
- [Standard library index](../README.md)
