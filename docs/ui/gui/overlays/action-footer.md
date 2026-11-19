# Action footer

Use an action footer to lay out the buttons at the bottom of a dialog. Available in **GUI**.

![Action footer in the FPAS gallery](images/action-footer.png)

## Example

```pascal
unit Docs.Ui.Overlays.ActionFooter;

uses Fpas.Ui.Elements as E,
  Fpas.Ui.Widgets.Actions.ActionFooters as W,
  Fpas.Ui.Widgets.Actions.Buttons as B, Fpas.Ui.Styles as S;

// Builds the action footer example.
// ## Returns
// The widget for the application view.
public function View(): E.Element<string>;
begin
  return W.ActionFooter<string>(
    B.Button<string>('cancel', 'Cancel', 'cancel'),
    B.Button<string>('save', 'Save changes', 'save', new B.Options(Variant := S.ButtonVariant.Primary)));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Children` | Pass the buttons in display order. |
| `Alignment / spacing` | The native row aligns right, with an eight-point top inset and an eight-point gap. |

## Interaction

The buttons retain their own activation messages and keyboard behavior. Apply the message in your update function to save or cancel and close the dialog.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Actions/ActionFooters.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Overlays/Widgets/Dialog.fpas)
- [Dialog](dialog.md)
- [Overlays](README.md)
