# Dialog

Use a dialog for a focused task that blocks input to the underlying view. Available in **GUI and TUI**.

![Dialog in the FPAS gallery](images/dialog.png)

## Example

```pascal
unit Docs.Ui.Overlays.Dialog;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Overlays.Dialogs as W;

// Builds the dialog example.
// ## Parameters
// - `Base`: The view below the overlay.
// - `Open`: Current visibility from the application model.
// ## Returns
// The widget for the application view.
public function View(Base: E.Element of string; Open: bool): E.Element of string;
begin
  return W.Dialog of string('edit-profile', Base, E.Text of string('Review the profile details.'), Open,
    new W.Options('Edit profile', Description := 'Changes are visible to your team.'),
    'close-profile');
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Title` | Required literal heading. |
| `Description` | Null omits the supporting text. In the GUI, an empty string reserves a blank line below the heading. |
| `Width` | 440 logical units. Zero uses the default. Both hosts fit the width to the viewport. |
| `Persistent / CloseButton` | false / true. Persistent suppresses Escape and scrim dismissal. |
| `InitialFocus` | null. Supply the ID of an enabled control inside the dialog to choose where focus starts. |
| `Draggable` | False. In the GUI, dragging the header moves the dialog within the window. |
| `Resizable` | False. In the GUI, dragging an edge or corner changes the dialog size. |

## Interaction

Click the close button, press Escape or click the scrim to send the dismiss
message. A focused text field gets Escape first. With Persistent enabled,
the close button still sends the message when shown. Closing the dialog restores focus to the preceding control.

In the TUI, put ordinary buttons in a row for actions. The dialog stays
centered when the terminal resizes and its body scrolls if space is limited.
In the GUI, keep the body available during the exit transition. CUI clips the
body if it cannot fit after a resize.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_dialog)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Overlays/Dialogs.fpas)
- [Terminal gallery](../../../../examples/ui/tui/gallery/README.md)
- [GUI gallery example](../../../../examples/ui/gui/gallery/Pages/Overlays/Widgets/Dialog.fpas)
- [Action footer](action-footer.md)
- [Overlays](README.md)
- [Terminal example](../../tui/overlays/dialog.md)

## Source files

- `examples/ui/gui/gallery/Pages/Overlays/Widgets/`
  - [DeleteDialog.fpas](../../../../examples/ui/gui/gallery/Pages/Overlays/Widgets/DeleteDialog.fpas)
- `src/Fpas.Ui.Gui/Internal/Widgets/Overlays/`
  - [Dialog.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Overlays/Dialog.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Overlays/`
  - [NativeMethods.Dialog.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Overlays/NativeMethods.Dialog.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Overlays/`
  - [CuiWindow.Dialog.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Overlays/CuiWindow.Dialog.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/overlays/`
  - [dialog.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/dialog.c)
  - [dialog.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/dialog.h)
- `tests/Fixtures/gui-host/`
  - [DialogContract.fpas](../../../../tests/Fixtures/gui-host/DialogContract.fpas)
  - [ModalCompositionContract.fpas](../../../../tests/Fixtures/gui-host/ModalCompositionContract.fpas)
  - [ModalPreflightContract.fpas](../../../../tests/Fixtures/gui-host/ModalPreflightContract.fpas)
- [CuiDialogTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Overlays/CuiDialogTests.cs)
