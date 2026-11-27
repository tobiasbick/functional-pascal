# Select

Use Select for one choice from a list. Enable search to filter a longer list inside its popup. Available in **GUI** and [TUI](../../tui/inputs/select.md).

![Select in the FPAS gallery](images/select.png)

## Example

```pascal
unit Docs.Ui.Inputs.Select;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Inputs.Selects as W;

// Builds the select example.
// ## Parameters
// - `Selected`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Selected: int): E.Element of int;
begin
  return W.Select of int('input-language', [new W.Item('C'), new W.Item('Zig'), new W.Item('Rust')],
    Selected, function(Index: int): int => Index,
    new W.Options(Label := 'Language', Placeholder := 'Pick a language', Searchable := true));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Items` | Options in model order. Each has a label and an optional icon. |
| `Selected` | Zero-based index, or -1 before a choice is made. |
| `Label` | Empty. Set a caption to display it above the trigger. |
| `Placeholder` | Null displays Select. An empty string leaves the unselected trigger blank. |
| `Searchable / Enabled` | false / true. |
| `Width` | Null uses CUI sizing. A labeled select fills its parent. An unlabeled select prefers 220 logical units. Set `Fill`, `Fixed`, or `Percent` to override the width. |

## Interaction

Click the trigger or press Enter, Space, Up, or Down when it has focus.
Choosing a row reports its original index even when search filters the list.
Escape dismisses the popup and returns focus to the trigger. Use
`OptionId(Id, Index)` to inspect a visible row.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_select)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/Selects.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Inputs/Widgets/PickerFields.fpas)
- [Inputs](README.md)

## Source files

- [Shared Country and Language examples](../../../../examples/ui/gui/gallery/Shared/PickerChoices.fpas)

- `src/Fpas.Ui.Gui/Internal/Widgets/Inputs/`
  - [Select.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Inputs/Select.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Inputs/`
  - [NativeMethods.Select.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Inputs/NativeMethods.Select.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.Select.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.Select.Types.cs)
  - [CuiWindow.Select.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.Select.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/inputs/`
  - [select.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/select.c)
  - [select.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/select.h)
