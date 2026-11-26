# Spacing

Use padding inside a container. Use a gap between adjacent children.
Use a spacer to allocate free space on the main axis.

## Gap and padding

| `Layout.Spacing` | Logical units in GUI |
| --- | ---: |
| `None` | 0 |
| `Tight` | 4 |
| `Normal` | 8 |
| `Loose` | 16 |
| `Fixed(Measure.Logical(...))` | The specified whole distance |
| `Fixed(Measure.Points(...))` | The specified fractional distance |

Set `Flow.Gap` to space all adjacent children in a row, column or grid.
A wrapping row applies the same gap between lines.
A grid applies that gap between its rows and columns.
A layered stack places its children in the same area, so it adds no gaps.

Set `Flow.Padding` to inset every edge. Set `PaddingX` or `PaddingY` to
override the inset on one axis. `PaddingTop`, `PaddingBottom`,
`PaddingStart` and `PaddingEnd` override the inset on a single edge.
The resolved reading direction maps the start and end edges to the
physical left and right edges. A null override inherits the fallback inset.
Use `Spacing.None` for an explicit zero inset.

## Fixed gaps and spacers

- Use `Elements.Gap(Distance)` to add a fixed child on the main axis.
  The child has zero extent on the cross axis.
- Use `Elements.Spacer()` to add a growing child. A row allocates width to
  the spacer. A column allocates height.
- Use `Elements.Size` to give a spacer an explicit growth weight or bounds.
  Without explicit weights, spacers share the remaining space equally.

A fixed gap or spacer counts as a child for `Flow.Gap`. A fixed gap of zero
still has a gap on each side when both neighboring children exist.
`Elements.Gap` and `Elements.Spacer` require the GUI capability profile.
`Flow.Gap` and `Flow.Padding` also work in TUI.

## See also

- [CUI API coverage](../../../cui-api-coverage.md#application-capabilities)
- [Layout gallery example](../../../../examples/ui/gui/gallery/Pages/Layout/View.fpas)
- [Container options](container-options.md)
- [Row](row.md)
- [Column](column.md)
- [Reading direction](reading-direction.md)
- [Shared sizing and measurement](../../layout/sizing.md)
- [GUI layout](README.md)

## Source files

- [Layout.fpas](../../../../src/Fpas.Ui/Layout.fpas)
- [Elements.fpas](../../../../src/Fpas.Ui/Elements.fpas)
- [GUI spacing and padding](../../../../src/Fpas.Ui.Gui/Internal/Layout/Sizing.fpas)
- [GUI presentation](../../../../src/Fpas.Ui.Gui/Internal/Presentation/Presentation.fpas)
- [Layout contract tests](../../../../tests/Fixtures/gui-host/LayoutContract.fpas)
