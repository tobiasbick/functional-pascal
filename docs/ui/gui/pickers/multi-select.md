# Multi-select

Use multi-select for independent choices from a fixed list. In CUI, the field
shows the selected items as chips and the popup shows checkboxes. Available in
**GUI** and [TUI](../../tui/inputs/multi-select.md).

![Multi-select in the FPAS gallery](images/multi-select.png)

## Example

```pascal
unit Docs.Ui.Pickers.MultiSelect;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Inputs.MultiSelects as W;
uses Fpas.Ui.Widgets.Inputs.Selects as S;

// Builds the multi-select example.
// ## Parameters
// - `Selected`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Selected: array of bool): E.Element of array of bool;
begin
  return W.MultiSelect of array of bool('picker-languages',
    [new S.Item('C'), new S.Item('Rust'), new S.Item('Swift')], Selected,
    function(Value: array of bool): array of bool => Value,
    new W.Options(Label := 'Languages', Searchable := true));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Items` | Supply ordered `Selects.Item` choices. Each choice can include a native glyph shown in the popup. |
| `Selected` | Supply one Boolean per item in the same order. |
| `Label` | Null omits the native label wrapper. An empty string keeps the wrapper without a caption. |
| `Placeholder` | Null uses Select for an empty selection. Pass an empty string to hide the placeholder. |
| `Searchable` | false. Enable to filter by text with ASCII case folding. |
| `Enabled` | true. Disabling the field closes the popup. |
| `MaxChips` | 0 shows up to three selected items as chips. Extra selections appear as a count. |
| `Width` | Null uses a soft 300-unit width without a label and fills the container with a label. Fill, Fixed, and Percent set the GUI width. |

## Interaction

Click the field to open the popup. Enter, Space, or Down also opens it from
the focused field. Up and Down move the highlight in the list. Enter or
Space toggles the highlighted choice. Space types a character when the search
field receives text input.

The popup stays open when you toggle a choice. Select all and Clear affect
the entire list. Remove a chip to deselect its item. Search preserves the
original indices and the selection of hidden choices. Accept a fresh Boolean
array in the model to apply each proposal.

Escape closes the popup and restores focus to the field.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_multi_select)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/MultiSelects.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Pickers/View.fpas)
- [Select](../inputs/select.md)
- [Pickers](README.md)

## Source files

- [Shared Languages example](../../../../examples/ui/gui/gallery/Shared/LanguagesExample.fpas)
- `src/Fpas.Ui.Gui/Internal/Widgets/Inputs/`
  - [MultiSelect.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Inputs/MultiSelect.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Inputs/`
  - [NativeMethods.MultiSelect.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Inputs/NativeMethods.MultiSelect.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.MultiSelect.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.MultiSelect.Types.cs)
  - [CuiWindow.MultiSelect.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.MultiSelect.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/inputs/`
  - [multi_select.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/multi_select.c)
  - [multi_select.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/multi_select.h)
