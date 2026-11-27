# Funnel

Use a funnel to show the amount that reaches each stage of a process. The gallery follows visits through subscription.

Available in **GUI**.

![Funnel in the FPAS gallery](images/funnel.png)

## Example

```pascal
unit Docs.Ui.MoreCharts.Funnel;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Charts.Charts as C;
uses Fpas.Ui.Widgets.Charts.Funnels as W;

// Builds the funnel example.
// ## Returns
// The widget for the application view.
public function View(): E.Element of string;
begin
  return W.Funnel of string('funnel', [
    new C.Item('Visited', 12840.0F), new C.Item('Signed up', 4210.0F),
    new C.Item('Activated', 2380.0F), new C.Item('Subscribed', 860.0F)
  ]);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Items` | Named nonnegative amounts in stage order with a finite sum. An item color sets that stage's fill. |
| `Width` | Fill. |
| `Color` | Automatic uses the theme accent or the first series color with a neutral accent. This color sets the base stage fill. |
| `Format` | Null uses native automatic formatting. |
| `Enabled` | True. Allows pointer inspection and activation. |

Height follows the stage count. CUI uses fixed stage and connector sizes
for every value of its native height field.

## Interaction

Hover a stage to inspect its amount and conversion rates. On a click, the
optional `OnActivate` callback receives the stage's zero-based input index.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_heatmap)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Charts/Funnels.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/MoreCharts/View.fpas)
- [Sankey](sankey.md)
- [More charts](README.md)

## Source files

- [Funnel.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Charts/Funnel.fpas)
- [CuiWindow.Funnel.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/CuiWindow.Funnel.cs)
- [funnel.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/funnel.c)
- [funnel.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/funnel.h)
