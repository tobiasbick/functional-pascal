# Split view

Use a split view for two panes with a draggable divider. The gallery shows
a notes pane and a detail pane.
Available in **GUI**.

![Two-pane split view in the FPAS gallery](images/split-view.png)

## Example

```pascal
unit Docs.Ui.AppChrome.SplitView;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Desktop.SplitViews as W;

// Shows two panes at the current split ratio.
// ## Parameters
// - `Ratio`: Model-owned share of the first pane.
// ## Returns
// The split with new ratios as messages.
public function View(Ratio: float): E.Element of float;
requires Ratio >= 0.0F and Ratio <= 1.0F;
begin
  return W.SplitView of float('notes-split', Ratio,
    E.Text of float('Notes'), E.Text of float('Selected note'),
    function(Value: float): float => Value,
    new W.Options(Height := L.Extent.Fixed(L.Measure.Logical(128)),
      MinimumFirst := 96.0F, MinimumSecond := 120.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id / Ratio` | Give the split a stable ID. Set the first pane's share to a value from 0 through 1. |
| `Width / Height` | Both Fill. CUI treats Content as Fill for these options. |
| `Vertical` | false for side-by-side panes. Set true to stack the panes. |
| `MinimumFirst / MinimumSecond` | Both 0. CUI uses 120 logical units for a zero minimum. Set a positive value to supply a minimum along the split axis. |

## Interaction

Drag the divider to propose a new ratio. Store the proposal in the model
and pass it to the next view. Use the same ratio to restore a saved split.

CUI reserves one logical unit for the divider. If the pane minimums exceed
the available space, CUI reduces them in proportion to each minimum. A
horizontal split follows the surrounding reading direction. A vertical split
puts its first pane at the top.

Close the first pane's child layouts before calling `NextSplitPane` in the
low-level CUI adapter. Close the second pane's child layouts before calling
`EndSplitView`.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_split)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Desktop/SplitViews.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Desktop/View.fpas)
- [Windowing tests](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Windowing/CuiWindowTests.cs)
- [Docking workspace](dockspace.md)
- [Sizing](../../layout/sizing.md)
- [Chapter index](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Desktop/`
  - [SplitView.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Desktop/SplitView.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Desktop/`
  - [NativeMethods.SplitView.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Desktop/NativeMethods.SplitView.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Desktop/`
  - [CuiWindow.SplitView.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Desktop/CuiWindow.SplitView.Types.cs)
  - [CuiWindow.SplitView.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Desktop/CuiWindow.SplitView.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/desktop/`
  - [split_view.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/desktop/split_view.c)
  - [split_view.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/desktop/split_view.h)
