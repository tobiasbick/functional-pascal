# Radar chart

Use a radar chart to compare several measurements for each of a few series on a common scale.

Available in **GUI**.

![Radar chart in the FPAS gallery](images/radar-chart.png)

## Example

```pascal
unit Docs.Ui.MoreCharts.RadarChart;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Charts.Charts as C;
uses Fpas.Ui.Widgets.Charts.RadarCharts as W;

// Builds the radar chart example.
// ## Returns
// The widget for the application view.
public function View(): E.Element of string;
begin
  return W.RadarChart of string('radar', [
    new C.Series('1.0', [82.0F, 64.0F, 90.0F]),
    new C.Series('1.1', [70.0F, 85.0F, 72.0F])
  ], ['Speed', 'Size', 'Polish'], new W.Options(Size := 220.0F, Maximum := 100.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Axes / Series` | At least three axis labels. Each series provides a name, one finite nonnegative value per axis in the same order, and an optional color. Automatic colors use the CUI series palette. |
| `Size` | 0. CUI uses a diameter of 260 logical points and can shrink the plot to fit a narrow container. |
| `Maximum` | 0. CUI selects the outer ring value from the data. Set a positive value for an explicit scale. |
| `Format` | null. CUI uses compact numbers. A value format applies to ring labels and hover values. |
| `Legend` | true. The legend shows when there are two or more series and wraps below the plot when space is narrow. |
| `Enabled` | true. Set false to disable pointer interaction. |

## Interaction

Hover near an axis to show the nearest series point and its value. The last series shows its points when the pointer leaves the chart. Supply `OnActivate` to receive the zero-based axis index when CUI reports activation.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_heatmap)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Charts/RadarCharts.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/MoreCharts/View.fpas)
- [Shared chart types](../../../../src/Fpas.Ui/Widgets/Charts/Charts.fpas)
- [More charts](README.md)

## Source files

- [RadarChart.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Charts/RadarChart.fpas)
- [CuiWindow.RadarChart.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/CuiWindow.RadarChart.cs)
- [radar_chart.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/radar_chart.c)
- [radar_chart.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/radar_chart.h)
