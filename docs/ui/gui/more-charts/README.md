# More charts

Many of these visualizations can send a message from a pointer selection.
The model supplies the input data and handles any resulting changes.
These widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Heatmap](heatmap.md) | Draw values in a numeric matrix. |
| [Calendar heatmap](calendar-heatmap.md) | Draw daily values in a calendar. |
| [Treemap](treemap.md) | Compare shares as rectangles. |
| [Radar chart](radar-chart.md) | Compare series across several axes. |
| [Funnel](funnel.md) | Show the amount at each stage of a process. |
| [Split bar](split-bar.md) | Divide one track into proportional segments. |
| [Waffle](waffle.md) | Divide a grid of squares into shares. |
| [Sankey](sankey.md) | Show where quantities flow between nodes. |
| [Bar list](bar-list.md) | Rank and animate named amounts. |
| [Activity rings](activity-rings.md) | Compare progress across several goals. |
| [Scatter chart](scatter-chart.md) | Plot two coordinates with optional bubble sizes. |
| [Radial gauge](radial-gauge.md) | Show a reading on a scale with optional thresholds or a target. |
| [Candlestick](candlestick.md) | Inspect price and volume across periods with zoom and pan. |

`Charts.Item` is shared by the categorical charts. It supplies a label, an
amount and a color. For click callbacks, preserve the original input order
when interpreting the returned index. Matrix and scatter callbacks instead
identify the coordinates or series and point described on their pages.

## More charts gallery page

The heatmaps appear first in separate cards. Treemap and Radar share a row
with growth weights of 1.3 and one. Funnel shares an equal-width row with
Split bar and Waffle. Sankey fills its own card.

Bar list and Activity rings share a row with growth weights of 1.4 and
one. Shuffle changes the six amounts and retains their labels. Scatter
fills a card. The three gauges wrap on narrow windows. The candlestick
chart shows 2,000 periods in the final card.

## See also

- [UI handbook](../README.md)
- [Charts](../charts/README.md)
- [More charts in the gallery](../../../../examples/ui/gui/gallery/Pages/MoreCharts/View.fpas)
- [CUI gallery comparison](../../../future/cui-compare/README.md#more-charts-evidence)

## Source files

<details>
<summary>Browse 22 source files</summary>

- `examples/ui/gui/gallery/Pages/MoreCharts/`
  - [Data.fpas](../../../../examples/ui/gui/gallery/Pages/MoreCharts/Data.fpas)
  - [Layout.fpas](../../../../examples/ui/gui/gallery/Pages/MoreCharts/Layout.fpas)
  - [Series.fpas](../../../../examples/ui/gui/gallery/Pages/MoreCharts/Series.fpas)
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/MoreCharts/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/MoreCharts/Update.fpas)
- `examples/ui/gui/gallery/Pages/MoreCharts/Widgets/`
  - [Flows.fpas](../../../../examples/ui/gui/gallery/Pages/MoreCharts/Widgets/Flows.fpas)
  - [Market.fpas](../../../../examples/ui/gui/gallery/Pages/MoreCharts/Widgets/Market.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Charts/`
  - [NativeMethods.MoreCharts.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Charts/NativeMethods.MoreCharts.Types.cs)
  - [NativeMethods.MoreCharts.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Charts/NativeMethods.MoreCharts.cs)
  - [NativeMethods.Visualizations.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Charts/NativeMethods.Visualizations.Types.cs)
  - [NativeMethods.Visualizations.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Charts/NativeMethods.Visualizations.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Charts/`
  - [ChartMemory.MoreCharts.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/ChartMemory.MoreCharts.cs)
  - [CuiMoreChartData.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/CuiMoreChartData.cs)
  - [CuiVisualizationData.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/CuiVisualizationData.cs)
  - [MoreChartValidation.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/MoreChartValidation.cs)
  - [VisualizationValidation.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/VisualizationValidation.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [more_charts.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/more_charts.c)
  - [visualizations.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/visualizations.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/charts/`
  - [more_charts.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/more_charts.h)
  - [visualizations.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/visualizations.c)
  - [visualizations.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/visualizations.h)
- `tests/Fpas.Ui.Gui.Cui.Tests/Display/`
  - [CuiMoreChartTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiMoreChartTests.cs)

</details>
