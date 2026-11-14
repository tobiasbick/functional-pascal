# Charts

Use line and bar charts to compare series, a donut or pattern meter to show
parts of a total, and a gauge or sparkline for a compact reading. These
widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Line chart](line-chart.md) | Show change across ordered samples, with optional area fills and smoothing. |
| [Bar chart](bar-chart.md) | Compare discrete categories with grouped or stacked bars. |
| [Donut chart](donut-chart.md) | Show a few shares as wedges. |
| [Pattern meter](pattern-meter.md) | Show used and free capacity with distinct patterns. |
| [Gauge](gauge.md) | Display one reading as a fraction from zero through one. |
| [Sparkline](sparkline.md) | Show a trend in a small space. |

Line and bar charts share series, labels, grid and legend settings, and
annotations. `Charts.Color.Automatic` uses the native default.
`Color.Palette(1..6)` follows the theme, and `Color.Rgba` supplies an explicit
color. A null number format uses CUI's automatic formatting.
`ValueFormat` supplies zero through six decimal places and literal prefix and
suffix strings. If automatic bounds cannot represent very large constant
values, supply a fixed range wide enough for the axis ticks to advance.

## See also

- [UI handbook](../README.md)
- [More charts](../more-charts/README.md)
- [Shared chart types](../../../src/Fpas.Ui/Widgets/Charts/Charts.fpas)
- [Charts in the gallery](../../../examples/ui/gui/gallery/Pages/Charts/View.fpas)
