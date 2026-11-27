# Sparkline

Use a sparkline for a compact trend, for example beside a metric.

See also the [terminal sparkline](../../tui/charts/sparkline.md).

![Sparkline sample](images/sparkline.png)

## Example

```pascal
unit Docs.Ui.Charts.Sparkline;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Charts.Sparklines as W;

// Builds the sparkline example.
// ## Parameters
// - `Values`: Samples in display order.
// ## Returns
// The widget for the application view.
public function View(Values: array of float): E.Element of string;
begin
  return W.Sparkline of string('chart-sparkline', Values, new W.Options(Height := 32.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Values` | Finite samples in display order. |
| `Width / Height` | Content / 0. Zero uses CUI's default height. |
| `Color` | Automatic. Use a theme color token or explicit RGBA color to customize the stroke. |

## Interaction

CUI draws the line and its endpoint without axes or a legend. The sparkline is passive. Use a line chart when the user needs labels and inspection.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_sparkline)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Charts/Sparklines.fpas)
- [Documentation capture](../../../../apps/gui-doc-captures/src/Pages/Charts.fpas)
- [Line chart](line-chart.md)
- [Charts](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Charts/`
  - [Sparkline.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Charts/Sparkline.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Charts/`
  - [NativeMethods.Sparkline.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Charts/NativeMethods.Sparkline.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Charts/`
  - [CuiWindow.Sparkline.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/CuiWindow.Sparkline.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/charts/`
  - [sparkline.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/sparkline.c)
  - [sparkline.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/sparkline.h)
