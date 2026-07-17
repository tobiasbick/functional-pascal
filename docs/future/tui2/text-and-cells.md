# Std.Tui2 text, cells, and palettes

Implemented color, style, cell-value, palette, and display-width behavior is documented in the
current [`Std.Tui2` reference](../../pascal/std/tui2/README.md) and
[`Std.Console` cell reference](../../pascal/std/console/cells-frames.md). This plan covers the
remaining grapheme-aware surface and canvas behavior.

## Width policy

- A renderable grapheme occupies one or two terminal columns.
- Ambiguous-width characters use width one.
- The runtime width policy is identical for live and headless rendering.
- Newline is a text-layout delimiter, not a cell glyph.
- Tab is expanded by the owning text control; the default tab width is four cells.
- Unsupported control characters render as the replacement glyph.
- A zero-width cluster without a preceding base glyph renders as the replacement glyph.

Measurement continues to use `Std.Console.DisplayWidth` so layout and final rendering cannot
disagree.

## Cell surface

The cell surface validates each `TuiCell` glyph as one non-empty renderable grapheme and enforces
wide-glyph continuation repair when it is painted.

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

`WriteText` draws one logical line and stops before a newline. It currently iterates Unicode
scalars, so a combined or joined grapheme must be passed through `PutCell` as a `TuiCell` until the
source library can segment text. Wrapping and multi-line placement belong to the calling control.
Every operation is clipped and uses the same wide-glyph repair rules.

## Palette propagation

Applications may replace a palette at the application or subtree boundary. A palette change
invalidates affected views. Color capability fallback occurs only in the final console renderer;
measurement and control logic remain color-mode independent.
