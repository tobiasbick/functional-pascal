# Radio button

Use an individual radio button when your view lays out the choices itself. The screenshot shows a selected native row in the gallery's radio group. Available in **GUI**.

![A selected native radio row in the FPAS gallery](images/radio-button.png)

## Example

```pascal
unit Docs.Ui.Selection.RadioButton;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Inputs.RadioButtons as C;

// Builds the radio button example.
// ## Parameters
// - `Selected`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Selected: bool): E.Element of string;
begin
  return C.RadioButton of string('save-periodic', 'Every 5 minutes', Selected,
    function(): string => 'periodic', Description := 'A good balance for most projects.');
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Selected` | Pass true for the choice currently held by the model. |
| `Description` | null omits the secondary text. An empty string reserves a blank description. |
| `Enabled` | true. |

## Interaction

Click the row or focus it and press Space or Enter. Activating an unselected
row sends its selection message. Keep one selection in the model and derive
each row's Boolean from it.

CUI uses the shared `cui_check_opts` type for checkboxes, switches and radio
buttons. Only the checkbox uses its `indeterminate` field.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_radio)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/RadioButtons.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Shared/SaveFrequency.fpas)
- [Radio group](radio-group.md)
- [Selection](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Inputs/`
  - [RadioButton.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Inputs/RadioButton.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Inputs/`
  - [NativeMethods.RadioButton.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Inputs/NativeMethods.RadioButton.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.RadioButton.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.RadioButton.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/inputs/`
  - [radio_button.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/radio_button.c)
  - [radio_button.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/radio_button.h)
