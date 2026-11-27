# Sheet and drawer

Use a sheet to open a modal panel from a window edge. The gallery uses a right sheet for Notifications and a left sheet as its compact navigation drawer. Available in **GUI and TUI**.

![Sheet and drawer in the FPAS gallery](images/sheet.png)

![Left navigation drawer at the gallery's compact width](images/drawer.png)

## Example

```pascal
unit Docs.Ui.Overlays.Sheet;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Overlays.Sheets as W;

// Builds the sheet and drawer example.
// ## Parameters
// - `Base`: The view below the overlay.
// - `Open`: Current visibility from the application model.
// ## Returns
// The widget for the application view.
public function View(Base: E.Element of string; Open: bool): E.Element of string;
begin
  return W.Sheet of string('notifications', Base, E.Text of string('Notification preferences'), Open,
    new W.Options('Notifications', Description := 'Choose what we tell you about.'),
    'close-notifications');
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Title` | Required heading. |
| `Description` | Null omits supporting text. In GUI, an empty string reserves a blank line. |
| `Edge` | Right. Use Left, Top or Bottom for another edge. |
| `Size` | 400 logical units. Zero uses the default. Set the width for Left or Right, or the height for Top or Bottom. |
| `InitialFocus` | null. The close button is the fallback focus target. |

## Interaction

The body scrolls beneath the pinned header while focus stays inside. The close button, backdrop and Escape propose dismissal. A focused editor gets Escape first. The host restores the previous focus when the model closes the sheet. GUI animates the panel. TUI lays it out in cells and supports Page Up and Page Down when the focused control leaves those keys available.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_sheet)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Overlays/Sheets.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Overlays/Widgets/Notifications.fpas)
- [Sidebar navigation](../navigation/sidebar.md)
- [Overlays](README.md)
- [Terminal example](../../tui/overlays/sheet.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Overlays/`
  - [Sheet.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Overlays/Sheet.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Overlays/`
  - [NativeMethods.Sheets.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Overlays/NativeMethods.Sheets.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Overlays/`
  - [CuiWindow.Sheets.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Overlays/CuiWindow.Sheets.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/overlays/`
  - [sheet.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/sheet.c)
  - [sheet.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/sheet.h)
