# Terminal rows and columns

Use `Elements.Row` to arrange children from left to right. Use
`Elements.Column` to arrange children from top to bottom. Both accept
`Layout.Flow` to set the gap, padding and cross-axis alignment.

## Cells and spacing

Use `Layout.Measure.Cells` for an exact terminal distance. Use
`Layout.Measure.Logical` for a portable distance. The TUI host rounds up
at eight logical units per column and 16 per row.

| `Layout.Spacing` | Horizontal cells | Vertical cells |
| --- | --- | --- |
| `None` | 0 | 0 |
| `Tight` | 1 | 1 |
| `Normal` | 1 | 1 |
| `Loose` | 2 | 1 |
| `Fixed(Measure.Cells(N))` | N | N |

`Flow.Gap` separates consecutive children along the main axis. `Flow.Padding`
insets all four sides. If padding uses logical units, the TUI host converts
the horizontal and vertical insets separately. Both settings default to `None`.

## Allocation and alignment

Use `Elements.Size` with `Layout.Flex` to constrain the main axis. In a row,
the rule applies to width. In a column, the rule applies to height. The TUI
host reserves space for padding and gaps before allocating cells to children.
Growing children share the remaining cells by weight and respect their bounds.

`Alignment.Default` centers children vertically in a row. In a column,
it stretches eligible children horizontally. Intrinsically sized controls,
such as regular buttons, keep their content width by default. Explicit
`Stretch` can make a button fill a column if the button has no fixed width.
Use `Start`, `Center` or `End` for cross-axis placement.

`Justification.Start` packs children from the main-axis start. Use the
[graphical profile](../../gui/layout/README.md) for other justification modes,
row wrapping, and native box decoration. Use a [scroll view](scroll-view.md)
to constrain a long column within the terminal viewport.

## See also

- [Measures, extents and size rules](../../layout/sizing.md)
- [Panels and weighted layout](panels.md)
- [Terminal viewport](viewport.md)
- [TUI gallery](../../../../examples/ui/tui/gallery/main.fpas)

## Source files

- [Layout.fpas](../../../../src/Fpas.Ui/Layout.fpas)
- [Elements.fpas](../../../../src/Fpas.Ui/Elements.fpas)
- [Flexible.fpas](../../../../src/Fpas.Ui.Tui/Internal/Layout/Flexible.fpas)
- [Measurement.fpas](../../../../src/Fpas.Ui.Tui/Internal/Layout/Measurement.fpas)
- [Stacks.fpas](../../../../src/Fpas.Ui.Tui/Internal/Layout/Stacks.fpas)
- [Lowering.fpas](../../../../src/Fpas.Ui.Tui/Internal/View/Lowering.fpas)
