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
`Color.Theme` resolves a color token against the active theme. `Color.Rgba`
supplies an explicit color. A null number format uses CUI's automatic formatting.
If automatic bounds cannot represent very large constant values, supply a
fixed range wide enough for the axis ticks to advance.

## Value format

`Charts.ValueFormat` configures a single floating-point value. It defaults to
fixed notation and zero decimal places.

| Option | Behavior |
| --- | --- |
| `Notation` | `Fixed`, `Scientific`, `General` or `Hexadecimal`. |
| `DecimalPlaces` | Precision from zero through 999. General notation counts significant digits. Set null for the notation's native precision. |
| `FieldWidth` | Minimum numeric width from zero through 999. |
| `Padding` | `LeadingSpaces`, `TrailingSpaces` or `LeadingZeros`. |
| `Sign` | `NegativeOnly`, `Always` or `Space`. |
| `Alternate` | Retain the decimal point and retain trailing zeros in general notation. Defaults to false. |
| `Uppercase` | Use uppercase letters in the exponent and hexadecimal digits. Defaults to false. |
| `Prefix` / `Suffix` | Literal text before and after the number. Percent signs remain literal. |

## Annotations

For line and bar charts, equal `XMinimum` and `XMaximum` use zero-based sample
indices. Set increasing bounds to place annotations in another X coordinate
system. For two or more bars, each bound marks the center of an end category.

Use `Annotation.Color` to set the shape and caption color. `Accent` applies
when `Color` is automatic. Null text omits its line. An empty string retains
the line's layout. CUI shows muted detail text on vertical lines and points.

## Charts gallery page

The revenue chart uses a smooth line and an area fill. A responsive grid
places the grouped visits bars beside the platform donut. Each grid column
contains its own heading and native card. The grid uses a 260-unit minimum
column width and a 16-unit gap.

The requests chart stacks two dither fills. The storage meter and its
Grayscale checkbox share a card with a gap of 24 logical units. The last
grid contains the stacked test-run bars and a centered row of two gauges.

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
