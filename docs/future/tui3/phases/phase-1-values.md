# Phase 1 — Values and owned rendering storage

Execution rules and the current baseline: [implementation phases](../implementation-phases.md).

## Task 1.1 — Complete geometry values

**Status:** complete.

**Files:** add `lib/Std/Tui3/Geometry/Point.fpas`; modify `Geometry/Size.fpas`,
`Geometry/Rect.fpas`, `lib/Std/Tui3.fpas`, and add
`tests/stdlib/tui3/geometry_values_test.fpas`.

**Contract:** implement [geometry.md](../geometry.md): zero-based points, non-negative sizes,
half-open rectangles, inset/intersection/containment without a second coordinate convention.
Port value semantics from `lib/Std/Tui2/Geometry/`, not its consumers.

**Done:** constructors reject illegal sizes; empty, edge, inset, intersection, and half-open
containment cases pass; the public facade exports all three values.

## Task 1.2 — Add cell, style, and palette values

**Status:** complete.

**Files:** add focused files under `lib/Std/Tui3/Cells/` for `Color`, `Style`, `StyleRole`, `Cell`,
and `Palette`; modify `lib/Std/Tui3.fpas`; add
`tests/stdlib/tui3/cell_palette_values_test.fpas`.

**Contract:** port the proven value semantics from `lib/Std/Tui2/Cells/` under Tui3 names and apply
[text-and-cells.md](../text-and-cells.md). A cell contains one validated renderable grapheme plus a
style; continuation cells remain private surface state. Palette lookup is semantic-role based.

**Do not touch:** `Std.Console` width behavior, the VM, or the existing Turbo Vision bridge.

**Done:** equality/copy behavior, palette resolution, invalid glyph diagnostics, and width-one/
width-two glyph values are covered.

## Task 1.3 — Replace the temporary row-string working surface

**Status:** complete.

**Files:** modify `Rendering/Surface.fpas`; add focused private files under
`lib/Std/Tui3/Rendering/Surface/` if cell repair would push the existing file past 400 LOC; add
`tests/stdlib/tui3/surface_cells_test.fpas` and
`tests/stdlib/tui3/surface_wide_glyph_test.fpas`.

**Contract:** the host-owned surface stores leading, continuation, and empty cells as specified in
[text-and-cells.md](../text-and-cells.md). Overwriting either half of a wide glyph repairs both
cells. Routine clear/write/paint does not construct `TuiSurfaceSnapshot`. Preserve the current
private slot ownership pattern until a separate language/runtime change explicitly replaces it.

**Done:** ASCII, clipping, wide glyph insertion/overwrite/right-edge behavior, clear, independent
hosts, snapshot immutability, and copied cell roles pass. Remove the Phase 0 row-string limitation from
[testing.md](../testing.md) when these tests pass.

## Task 1.4 — Add the private clipped canvas

**Status:** complete.

**Files:** add `Rendering/Canvas.fpas`; modify `Rendering/Paint.fpas`; add
`tests/stdlib/tui3/canvas_clip_test.fpas`.

**Contract:** the canvas is a private frame-scoped capability over one working surface. It accepts
local zero-based coordinates, applies origin and clip, exposes glyph/text/fill/frame operations,
and never exposes continuation cells or a copied grid. `WriteText` stops at newline and uses
`Std.Console.DisplayWidth`.

**Done:** nested origins and clips cannot paint outside their intersection; paint uses the canvas
instead of writing surface coordinates directly.

## Phase checkpoint

Tasks 1.1–1.4 are complete, the Phase 0 tests still pass, and no public mutable cell grid or
retained view handle exists.
