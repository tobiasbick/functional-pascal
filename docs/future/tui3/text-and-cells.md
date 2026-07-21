# Std.Tui3 text, cells, and palettes

Color, style, cell, palette, and display-width behavior should match the proven Tui2 and
console contracts:

- [`Std.Tui2` cell values](../../pascal/std/tui2/README.md) (as salvage reference)
- [`Std.Console` cells](../../pascal/std/console/cells-frames.md)

Tui3 owns the same rules under the `Tui` prefix inside `Std.Tui3`. It does not invent a
second width policy.

## Width policy

- A renderable grapheme occupies one or two terminal columns.
- Ambiguous-width characters use width one.
- Headless and live rendering share the same width policy.
- Newline is a text-layout delimiter, not a cell glyph.
- Tab expansion belongs to text controls; default tab width is four cells.
- Unsupported control characters and stranded zero-width clusters use a replacement glyph.

Measurement uses `Std.Console.DisplayWidth` so layout and paint cannot disagree.

## Cell surface

The application host owns one mutable working surface. It validates each `TuiCell` glyph as one
non-empty renderable grapheme and enforces wide-glyph continuation repair when painted. The working
surface is not a public pass-by-value record.

Internal cell kinds:

```text
empty cell
leading cell with glyph and style
continuation cell owned by a two-column glyph
```

Overwriting either column of a wide glyph clears the complete previous glyph. A wide glyph
paints only when both columns lie inside the clip; otherwise the visible portion becomes a
normal blank.

An internal frame-scoped canvas accepts local coordinates, clips every operation, and never exposes
continuation cells. `WriteText` draws one logical line, stops before a newline, and segments extended
grapheme clusters before painting. A future custom-paint element may receive this capability only
after the Phase 0 ownership and clone gate passes.

`TuiApplication.SurfaceSnapshot` explicitly copies the most recently painted cells into an
immutable `TuiSurfaceSnapshot` for tests. Snapshot construction is allowed to be proportional to the
cell count; ordinary layout and paint must not construct or copy snapshots.

## Palette

`TuiPalette` maps semantic `TuiStyleRole` values to styles. Applications may choose a
palette per frame through the model or a host setting. Color capability fallback happens
only in the final console renderer; measurement stays color-mode independent.

## Paint pipeline

Each frame:

1. clear or reuse the application surface for the current size;
2. walk the laid-out element tree back to front;
3. paint chrome and controls through canvases clipped to resolved rectangles;
4. flush the working surface through `Std.Console` when interactive;
5. create a snapshot only when a test explicitly requests one.

There is no per-widget retained dirty region API in the application model. Internal damage tracking
is an optimization only. Phase 0 must verify that clearing, canvas calls, and snapshot-free frames
do not repeatedly clone the complete cell grid.
