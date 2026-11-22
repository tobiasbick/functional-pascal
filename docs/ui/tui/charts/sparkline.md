# Sparkline

The **Components → Charts** page shows the CUI sequence
`3, 5, 4, 8, 6, 9, 8` at its natural width and in a three-row plot.

Use `Sparklines.Sparkline` to show how a finite series changes over time or
another ordered dimension. Each terminal row provides eight levels. The
sparkline scales relative to the smallest and largest sample. Constant
series use a middle level, and empty series leave the plot blank.

Natural width uses one column per sample when space permits. Fixed and
fill widths interpolate the values across the allocated columns. A height
of zero uses one row. Positive logical heights round up to whole rows.
`Color` takes a theme series or an explicit color. The plot has no focus stop.

## See also

- [Shared API](../../widgets.md#charts)
- [GUI sparkline](../../gui/charts/sparkline.md)

## Source files

- [Sparkline.fpas](../../../../src/Fpas.Ui.Tui/Internal/Widgets/Charts/Sparkline.fpas)
- [PlotSamples.fpas](../../../../examples/ui/tui/gallery/PlotSamples.fpas)
