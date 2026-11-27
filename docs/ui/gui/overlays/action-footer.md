# Action footer

Use an action footer to lay out the buttons at the bottom of a dialog. Available in **GUI**.

![Action footer in the FPAS gallery](images/action-footer.png)

## Example

```pascal
unit Docs.Ui.Overlays.ActionFooter;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Styles as S;
uses Fpas.Ui.Widgets.Actions.ActionFooters as W;
uses Fpas.Ui.Widgets.Actions.Buttons as B;

// Builds the action footer example.
// ## Returns
// The widget for the application view.
public function View(): E.Element of string;
begin
  return W.ActionFooter of string(
    B.Button of string('cancel', 'Cancel', 'cancel'),
    B.Button of string('save', 'Save changes', 'save', new B.Options(Variant := S.ButtonVariant.Primary)));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Children` | Pass the buttons in display order. |
| `Alignment / spacing` | CUI aligns the row to the trailing edge. The top inset and gap are 8 logical units. |

## Interaction

The buttons retain their activation messages and keyboard behavior. Apply the
message in your update function to save or cancel and close the dialog. If the
user resizes the dialog, the native footer stays at the bottom.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_dialog)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Actions/ActionFooters.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Overlays/Widgets/Dialog.fpas)
- [Dialog](dialog.md)
- [Overlays](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Actions/`
  - [ActionFooter.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Actions/ActionFooter.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Actions/`
  - [NativeMethods.ActionFooter.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Actions/NativeMethods.ActionFooter.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Actions/`
  - [CuiWindow.ActionFooter.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Actions/CuiWindow.ActionFooter.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/actions/`
  - [action_footer.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/action_footer.c)
  - [action_footer.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/action_footer.h)
