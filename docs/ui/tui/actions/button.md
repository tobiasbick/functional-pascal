# Button

Run a command such as saving the current draft. Open **Components → Buttons** for the shared variants, sizes, icons and states.

![Button rendered in terminal cells](../../../../examples/ui/tui/gallery/buttons.png)

## Example

```pascal
unit Docs.Tui.Actions.Button;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Styles as S;
uses Fpas.Ui.Widgets.Actions.Buttons as B;

// Builds the button example.
// ## Returns
// The control for this view.
public function View(): E.Element of string;
begin
  return B.Button of string('save', 'Save changes', 'save', new B.Options(Variant := S.ButtonVariant.Primary));
end function;

end unit;
```

## Interaction

Tab to the button and press Enter or Space, or click it. Set `Enabled := false` when the action is unavailable. The button keeps its ID when the caption changes.

## See also

- [Actions](README.md)
- [Shared API](../../widgets.md#button)
- [GUI button](../../gui/actions/button.md)

## Source files

- [Buttons.fpas](../../../../src/Fpas.Ui.Tui/Internal/Rendering/Buttons.fpas)
- [Button.fpas](../../../../src/Fpas.Ui.Tui/Internal/Widgets/Actions/Button.fpas)
