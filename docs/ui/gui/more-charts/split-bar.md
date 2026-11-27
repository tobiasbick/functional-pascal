# Split bar

Use a split bar to show how categories divide a total along one track.

Available in **GUI**.

![Split bar in the FPAS gallery](images/split-bar.png)

## Example

```pascal
unit Docs.Ui.MoreCharts.SplitBar;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Charts.Charts as C;
uses Fpas.Ui.Widgets.Charts.SplitBars as W;

// Builds the split bar example.
// ## Returns
// The widget for the application view.
public function View(): E.Element of string;
begin
  return W.SplitBar of string('split', [
    new C.Item('Layout', 38.0F), new C.Item('Text', 24.0F), new C.Item('Paint', 21.0F)
  ], new W.Options(Format := new C.ValueFormat(Suffix := ' ms')));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Items` | Named nonnegative amounts with a finite sum. Each item can specify a segment color. |
| `Width` | Fill. |
| `Height` | Zero uses a bar thickness of 10 logical units. |
| `Format` | Null uses native automatic formatting. |
| `Legend` | True. Shows values and shares in a wrapping legend below the bar. |
| `Enabled` | True. Allows pointer inspection and activation. |

## Interaction

Hover a segment or legend entry to inspect the amount and share. The optional
`OnActivate` callback receives the item's zero-based input index when clicked.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_heatmap)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Charts/SplitBars.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/MoreCharts/View.fpas)
- [Donut chart](../charts/donut-chart.md)
- [Pattern meter](../charts/pattern-meter.md)
- [More charts](README.md)

## Source files

- [SplitBar.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Charts/SplitBar.fpas)
- [CuiWindow.SplitBar.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/CuiWindow.SplitBar.cs)
- [split_bar.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/split_bar.c)
- [split_bar.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/split_bar.h)
