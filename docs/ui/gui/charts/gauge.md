# Gauge

Use a gauge for a single reading on a normalized scale. The gallery shows CPU and memory utilization.

Available in **GUI**.

![Gauge in the FPAS gallery](images/gauge.png)

## Example

```pascal
unit Docs.Ui.Charts.Gauge;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Charts.Gauges as W;

// Builds the gauge example.
// ## Parameters
// - `Fraction`: Current reading from zero through one.
// ## Returns
// The widget for the application view.
public function View(Fraction: float): E.Element of string;
begin
  return W.Gauge of string('cpu', 'CPU', Fraction, 130.0F);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | Stable gauge ID. Keep it when the caption changes. |
| `Label` | Literal caption. Pass null to omit it. |
| `Value` | Finite reading. CUI clamps the displayed value to the range 0 through 1. |
| `Size` | Finite diameter in logical units. Zero or a negative value uses 140. |

## Interaction

CUI animates the arc and displays the reading as a whole percentage. The arc uses the warning color above 75% and the danger color above 90%. The gauge is passive. Use a radial gauge for a scale in physical units, threshold bands or a target marker.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_gauge)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Charts/Gauges.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Charts/View.fpas)
- [Radial gauge](../more-charts/radial-gauge.md)
- [Progress ring](../feedback/progress-ring.md)
- [Charts](README.md)

## Source files

- [Gauge.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Charts/Gauge.fpas)
- [CuiWindow.Gauge.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/CuiWindow.Gauge.cs)
- [gauge.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/gauge.c)
- [gauge.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/gauge.h)
