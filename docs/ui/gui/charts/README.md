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
- [Shared chart types](../../../../src/Fpas.Ui/Widgets/Charts/Charts.fpas)
- [Charts in the gallery](../../../../examples/ui/gui/gallery/Pages/Charts/View.fpas)

## Source files

<details>
<summary>Browse 14 source files</summary>

- `examples/ui/gui/gallery/Pages/Charts/`
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/Charts/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Charts/Update.fpas)
- `src/Fpas.Ui.Gui.Cui/Hosting/`
  - [CuiWindow.AxisLabels.cs](../../../../src/Fpas.Ui.Gui.Cui/Hosting/CuiWindow.AxisLabels.cs)
- `src/Fpas.Ui.Gui.Cui/Interop/Charts/`
  - [NativeMethods.Charts.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Charts/NativeMethods.Charts.Types.cs)
  - [NativeMethods.Charts.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Charts/NativeMethods.Charts.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Charts/`
  - [ChartMemory.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/ChartMemory.cs)
  - [ChartValidation.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/ChartValidation.cs)
  - [CuiChartData.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/CuiChartData.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [charts.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/charts.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/charts/`
  - [charts.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/charts.c)
  - [charts.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/charts.h)
- `tests/Fpas.Ui.Gui.Cui.Tests/Display/`
  - [CuiAxisLabelTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiAxisLabelTests.cs)
  - [CuiChartTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiChartTests.cs)
  - [CuiVisualizationTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiVisualizationTests.cs)

</details>
