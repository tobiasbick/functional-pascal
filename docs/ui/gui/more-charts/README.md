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

## See also

- [UI handbook](../README.md)
- [Charts](../charts/README.md)
- [More charts in the gallery](../../../../examples/ui/gui/gallery/Pages/MoreCharts/View.fpas)
