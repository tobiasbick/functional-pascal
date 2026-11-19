# Sparkline

Use a sparkline for a compact trend, for example beside a metric.

Available in **GUI**.

![Sparkline in the FPAS gallery](images/sparkline.png)

## Example

```pascal
unit Docs.Ui.Charts.Sparkline;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Charts.Sparklines as W;

// Builds the sparkline example.
// ## Parameters
// - `Values`: Samples in display order.
// ## Returns
// The widget for the application view.
public function View(Values: array of float): E.Element<string>;
begin
  return W.Sparkline<string>('chart-sparkline', Values, new W.Options(Height := 32.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Values` | Finite samples in display order. |
| `Width / Height` | Content / 0. Zero uses CUI's default height. |
| `Color` | Automatic. Use a theme palette index or explicit RGBA color to customize the stroke. |

## Interaction

CUI draws the line and its endpoint without axes or a legend. The sparkline is passive. Use a line chart when the user needs labels and inspection.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Charts/Sparklines.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Charts/View.fpas)
- [Line chart](line-chart.md)
- [Charts](README.md)
