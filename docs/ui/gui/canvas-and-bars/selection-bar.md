# Selection bar

Use a selection bar for bulk operations. The gallery shows a selected
file count and actions beneath the file list.
Available in **GUI**.

![Files with the bulk selection bar in the FPAS gallery](images/selection-bar.png)

## Example

```pascal
unit Docs.Ui.CanvasAndBars.SelectionBar;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Canvas.SelectionBars as W;

// Adds a bulk action bar to a list.
// ## Parameters
// - `Child`: List that anchors the floating bar.
// - `Selected`: Selection flags in the list's order.
// - `Actions`: Action glyphs, captions and tones in display order.
// ## Returns
// The list with bulk action and clear proposals.
public function View(
  Child: E.Element of W.Event;
  Selected: array of bool;
  Actions: array of W.Action
): E.Element of W.Event;
begin
  return W.SelectionBar of W.Event('bulk-files', W.Selection.Items(Selected), Actions,
    function(Change: W.Event): W.Event => Change, Child);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Selected / Actions` | Supply `Selection.Items(Flags)` or `Selection.Count(Count)` and ordered actions. The count must be nonnegative. |
| `NoEscape / Enabled` | false / true. NoEscape keeps selection when Escape is pressed. |
| `CountFormat` | Empty uses CUI's default. At most one `%d` slot and literal `%%` are accepted. |

## Interaction

Use `Event.Index` to resolve the action and apply it to the selected
items. When `Clear` is true, clear the flags or reset the aggregate count in
the model. Count mode needs no item-flag array and retains native count animation.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_selection_bar)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Canvas/SelectionBars.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Canvas/View.fpas)
- [Canvas and bars](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Canvas/`
  - [SelectionBar.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Canvas/SelectionBar.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Canvas/`
  - [NativeMethods.SelectionBar.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Canvas/NativeMethods.SelectionBar.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Canvas/`
  - [CuiWindow.SelectionBar.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Canvas/CuiWindow.SelectionBar.Types.cs)
  - [CuiWindow.SelectionBar.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Canvas/CuiWindow.SelectionBar.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/canvas/`
  - [selection_bar.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/canvas/selection_bar.c)
  - [selection_bar.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/canvas/selection_bar.h)
