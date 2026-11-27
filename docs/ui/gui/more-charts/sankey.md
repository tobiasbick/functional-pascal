# Sankey

Use a Sankey diagram to show how amounts flow among named nodes. The gallery traces sign-ups from referral sources to outcomes.

Available in **GUI**.

![Sankey in the FPAS gallery](images/sankey.png)

## Example

```pascal
unit Docs.Ui.MoreCharts.Sankey;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Charts.Sankeys as W;

// Builds the sankey example.
// ## Returns
// The widget for the application view.
public function View(): E.Element of W.Selection;
begin
  return W.Sankey of W.Selection('flow',
    [new W.Node('Search'), new W.Node('Sign up'), new W.Node('Trial'), new W.Node('Paid')],
    [new W.Link(0, 1, 420.0F), new W.Link(1, 2, 420.0F), new W.Link(2, 3, 260.0F)],
    new W.Options(Height := 280.0F), function(Item: W.Selection): W.Selection => Item);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Nodes` | Named nodes with optional colors. Automatic colors use neutral gray for sources and intermediate nodes and palette colors for outcomes. |
| `Links` | Use distinct `From` and `To` indices into the node array. Amounts must be finite and nonnegative. Zero hides a link. |
| `Width / Height / NodeWidth` | Fill / 0 / 0. The native defaults are 300 points high and 6 points wide for each node bar. |
| `Format / Enabled` | null / true. |

## Interaction

Hover over a node or flow to highlight its path and show its amounts. A click can send `Selection.Node(Index)` or `Selection.Link(Index)`. Indices refer to the input arrays, including the positions of hidden zero-amount links.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_sankey)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Charts/Sankeys.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/MoreCharts/Widgets/Flows.fpas)
- [Funnel](funnel.md)
- [More charts](README.md)

## Source files

- [Sankey.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Charts/Sankey.fpas)
- [CuiWindow.Sankey.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Charts/CuiWindow.Sankey.cs)
- [sankey.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/sankey.c)
- [sankey.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/charts/sankey.h)
