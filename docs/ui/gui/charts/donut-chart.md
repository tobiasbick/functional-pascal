# Donut chart

Use a donut to show a small set of shares as parts of a total.

Available in **GUI**.

![Donut chart in the FPAS gallery](images/donut-chart.png)

## Example

```pascal
unit Docs.Ui.Charts.DonutChart;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Charts.Charts as C;
uses Fpas.Ui.Widgets.Charts.DonutCharts as W;

// Builds the donut chart example.
// ## Returns
// The widget for the application view.
public function View(): E.Element of string;
begin
  return W.DonutChart of string('platforms', [46.0F, 27.0F, 17.0F, 10.0F],
    ['Windows', 'macOS', 'Linux', 'Other'],
    new W.Options(Height := 180.0F, Format := new C.ValueFormat(Suffix := '%')));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | A stable chart ID. |
| `Values` | Finite nonnegative shares with a finite sum. |
| `Labels` | An empty array or one literal label per share. An empty array omits the legend. |
| `Width` | `Content`. A native width can also fill, specify a fixed width, or use a percentage. |
| `Height` | Preferred ring diameter in logical units. Zero uses CUI's default of 180. |
| `Legend` | true. Set false to hide the legend while retaining center captions. |
| `Format` | null uses CUI's automatic format for the center value. See [value format](README.md#value-format). |

## Interaction

CUI draws wedges proportional to the shares. Hover a wedge to show its value
and caption in the center. At rest, the center shows the total. If a format
contains a literal percent sign, the center shows the largest share at rest.
Use values already expressed in percent for that format.

The legend computes each percentage from the submitted shares and their
total. This calculation is independent of `Format`. CUI places the legend
below the ring when the chart is too narrow to fit both side by side.

For a stacked legend, CUI sizes the ring as if at least 120 logical units
were available, up to the preferred diameter. Without a legend, CUI applies
an 80-unit minimum in the narrow layout. A fixed chart width below these
limits can leave the ring wider than the chart's bounds.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_line_chart)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Charts/DonutCharts.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Charts/View.fpas)
- [Split bar](../more-charts/split-bar.md)
- [Charts](README.md)

## Source files

- [DonutChart.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Charts/DonutChart.fpas)
- [CuiWindow.DonutChart.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/CuiWindow.DonutChart.cs)
- [donut_chart.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/donut_chart.c)
- [donut_chart.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/donut_chart.h)
