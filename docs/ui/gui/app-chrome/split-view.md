# Split view

App chrome shows a notes pane and a detail pane with a draggable divider.
Available in **GUI**.

![Two-pane split view in the FPAS gallery](images/split-view.png)

## Example

```pascal
unit Docs.Ui.AppChrome.SplitView;

uses Fpas.Ui.Elements as E, Fpas.Ui.Layout as L, Fpas.Ui.Widgets.Desktop.SplitViews as W;

// Shows two panes at the current split ratio.
// ## Parameters
// - `Ratio`: Model-owned share of the first pane.
// ## Returns
// The split with new ratios as messages.
public function View(Ratio: float): E.Element<float>;
requires Ratio >= 0.0F and Ratio <= 1.0F;
begin
  return W.SplitView<float>('notes-split', Ratio,
    E.Text<float>('Notes'), E.Text<float>('Selected note'),
    function(Value: float): float => Value,
    new W.Options(Height := L.Extent.Fixed(L.Measure.Logical(128)),
      MinimumFirst := 96.0F, MinimumSecond := 120.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Width / Height` | Fill / Fill. |
| `Vertical` | false. Set true to stack the panes. |
| `MinimumFirst / MinimumSecond` | 0 / 0. Nonzero values are logical points along the split axis. |

## Interaction

Drag the divider to propose a new ratio. Accept the proposal in the
update and pass it back to the next view. Use the same ratio when restoring
a saved split.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Desktop/SplitViews.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Desktop/View.fpas)
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
