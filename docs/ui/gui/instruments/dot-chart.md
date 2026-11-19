# Dot-matrix chart

Use a dot-matrix chart for equally spaced samples in stacked bands.
The gallery plots each cup's estimated caffeine over time and marks the
threshold, peak and bedtime. Available in **GUI**.

![Dot-matrix chart in the FPAS gallery](images/dot-chart.png)

## Example

```pascal
unit Docs.Ui.Instruments.DotChart;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Charts.DotCharts as W,
  Fpas.Ui.Widgets.Charts.Charts as C;

// Messages from the chart.
public type Msg = variant
  // Request a new event at the activated X coordinate.
  AddCup(Hit: W.Hit);
end variant;

// Displays five equally spaced samples for each of two cups.
// ## Returns
// The chart for the application view.
public function View(): E.Element<Msg>;
begin
  const Bands: array of W.Series := [
    new W.Series([0.0F, 80.0F, 62.0F, 42.0F, 30.0F]),
    new W.Series([0.0F, 0.0F, 30.0F, 32.0F, 24.0F], Accent := true)
  ];
  return W.DotChart<Msg>('caffeine', Bands, 5, Msg.AddCup,
    new W.Options(XMinimum := 6.0, XMaximum := 30.0, Units := W.AxisUnits.Hours,
      Range := new C.ValueRange(0.0F, 120.0F), Unit := 'MG',
      Annotations := [new C.Annotation(C.AnnotationKind.Y, Y := 50.0, Label := '50')]));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Bands / Count` | Supply up to 16 bands with `Count` finite, equally spaced samples in each. Pass the intended sample count for an empty plot. |
| `XMinimum / XMaximum / Units` | 0 / 0 / Number. Equal bounds use sample indices. Hours formats 12.25 as 12:15 and wraps at midnight. |
| `XFormat / XStep` | null / 0. Zero chooses automatic X label spacing. |
| `Range / Format / Unit` | null / null / empty. The chart derives value bounds and formatting when omitted. |
| `XLabel / ValueLabel` | Both null. Optional callbacks format X-axis and readout coordinates or the readout value during painting. |
| `Width / Height / Pitch` | Fill / 0 / 0. Zero uses native height or dot spacing. |
| `Grid / Axis / Readout / Enabled` | All true. |
| `Annotations` | Empty. Reference lines and points use axis units. |

## Interaction

Activation reports an X coordinate, the nearest sample index and the stacked
value at that sample. The index is minus one outside the plot. The gallery
uses the coordinate to add a cup and updates the plotted bands in its model.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Charts/DotCharts.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Instruments/View.fpas)
- [Event track](event-track.md)
- [Instruments](README.md)
