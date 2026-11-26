# List item

Use a list item for a row with a title and optional secondary information. Available in **GUI**.

![List item in the FPAS gallery](images/list-item.png)

## Example

```pascal
unit Docs.Ui.DataDisplay.ListItem;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Collections.ListItems as W;

// Builds the list item example.
// ## Parameters
// - `Selected`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Selected: bool): E.Element of string;
begin
  return W.ListItem of string('design-review', 'Design review',
    function(): string => 'open-review',
    new W.Options(Subtitle := 'Tomorrow at 10:00', Avatar := 'Jonas Berg', Trailing := '45m',
      Selected := Selected));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Subtitle / Trailing` | Empty. Use Trailing for short text such as a duration. |
| `Avatar / Icon` | Empty. Supply a name to display initials or a CUI glyph for an icon. |
| `Selected / Enabled` | false / true. |
| `OnActivate` | Optional message factory. |

## Interaction

An enabled row with an action accepts a click, Enter or Space while focused. The application decides whether to change selection. Omit the callback for a passive row like the gallery's examples.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_list_item)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Collections/ListItems.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Lists/Widgets/TreeAndList.fpas)
- [Terminal example using the same list data](../../tui/data-display/selection-list.md)
- [Tree](tree.md)
- [Avatar](../disclosure/avatar.md)
- [Data display](README.md)

## Source files

- [Shared CUI list entries](../../../../examples/ui/gui/gallery/Shared/ListEntries.fpas)
- `src/Fpas.Ui.Gui/Internal/Widgets/Collections/`
  - [ListItem.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Collections/ListItem.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Collections/`
  - [NativeMethods.ListItem.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Collections/NativeMethods.ListItem.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Collections/`
  - [CuiWindow.ListItem.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.ListItem.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/collections/`
  - [list_item.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/list_item.c)
  - [list_item.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/list_item.h)
