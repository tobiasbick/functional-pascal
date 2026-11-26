# Activity rings

Use concentric activity rings to compare progress on several goals.

Available in **GUI**.

![Activity rings in the FPAS gallery](images/activity-rings.png)

## Example

```pascal
unit Docs.Ui.MoreCharts.ActivityRings;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Charts.ActivityRings as W;
uses Fpas.Ui.Widgets.Charts.Charts as C;

// Builds the activity rings example.
// ## Returns
// The widget for the application view.
public function View(): E.Element of string;
begin
  return W.ActivityRings of string('rings', [
    new C.Item('Move', 0.82F), new C.Item('Exercise', 1.24F), new C.Item('Stand', 0.58F)
  ], new W.Options(Size := 170.0F, Center := '82%', Caption := 'of goal'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Items` | Up to eight labeled nonnegative progress fractions. |
| `Size / Thickness` | 0 / 0. CUI chooses the diameter and ring thickness. |
| `Center / Caption / Legend / Enabled` | Empty / empty / false / true. Center and Caption are literal text. |

## Interaction

CUI keeps drawing beyond one revolution when a value exceeds 1, up to four turns. Hover to inspect a ring. An optional `OnActivate` callback receives the clicked ring's input index.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_sankey)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Charts/ActivityRings.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/MoreCharts/Widgets/Flows.fpas)
- [Progress ring](../feedback/progress-ring.md)
- [More charts](README.md)

## Source files

- [ActivityRings.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Charts/ActivityRings.fpas)
- [CuiWindow.ActivityRings.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/CuiWindow.ActivityRings.cs)
- [activity_rings.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/activity_rings.c)
- [activity_rings.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/activity_rings.h)
