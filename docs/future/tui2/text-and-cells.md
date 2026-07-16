# Std.Tui2 text, cells, and palettes

## Text unit

Std.Tui2 lays out terminal text by extended grapheme cluster rather than by byte or Unicode scalar count. The shared runtime text boundary must expose deterministic grapheme segmentation and display width; this is a generic text/console capability, not widget logic.

The existing console cell API must be extended from one Unicode scalar to one non-empty grapheme cluster before Tui2 text rendering is complete.

## Width policy

- A renderable grapheme occupies one or two terminal columns.
- Ambiguous-width characters use width one.
- The runtime width policy is identical for live and headless rendering.
- Newline is a text-layout delimiter, not a cell glyph.
- Tab is expanded by the owning text control; the default tab width is four cells.
- Unsupported control characters render as the replacement glyph.
- A zero-width cluster without a preceding base glyph renders as the replacement glyph.

The public measurement operation is shared with `Std.Console.DisplayWidth` so layout and final rendering cannot disagree.

## Cell representation

The internal surface distinguishes:

```text
empty cell
leading cell with glyph and style
continuation cell owned by a two-column glyph
```

Overwriting either column of a wide glyph clears the complete previous glyph before painting the replacement. A wide glyph is painted only when both columns are inside the active clip. Otherwise the visible portion is filled with a normal blank cell.

`TuiCanvas` accepts local coordinates and clips every operation. It never exposes continuation cells to application paint handlers.

## Initial canvas operations

```pascal
TuiCanvas.PutCell(Canvas: TuiCanvas; Position: TuiPoint; Cell: TuiCell)
TuiCanvas.FillRect(Canvas: TuiCanvas; Bounds: TuiRect; Cell: TuiCell)
TuiCanvas.WriteText(Canvas: TuiCanvas; Position: TuiPoint; Text: string; Role: TuiStyleRole)
TuiCanvas.DrawHorizontalLine(Canvas: TuiCanvas; X: integer; Y: integer; Width: integer; Role: TuiStyleRole)
TuiCanvas.DrawVerticalLine(Canvas: TuiCanvas; X: integer; Y: integer; Height: integer; Role: TuiStyleRole)
TuiCanvas.DrawFrame(Canvas: TuiCanvas; Bounds: TuiRect; Role: TuiStyleRole)
```

`WriteText` draws one logical line and stops before a newline. Wrapping and multi-line placement belong to the calling control. Every operation is clipped and uses the same wide-glyph repair rules.

## Semantic styling

Controls paint semantic `TuiStyleRole` values instead of fixed colors. `TuiPalette` maps roles to foreground, background, and attributes.

Initial roles include normal, disabled, focused, selected, shortcut, frame, title, error, warning, and accent. Applications may replace a palette at the application or subtree boundary. A palette change invalidates affected views.

Color values support classic colors, indexed colors, and RGB. Capability fallback occurs only in the final console renderer; measurement and control logic are color-mode independent.

## Phase 1 prerequisite

Phase 1 adds or exposes the generic grapheme and cell primitives required by this contract before custom controls depend on them. Std.Tui2 itself remains FPAS source code.
