# Switch

Use a switch for an on/off setting such as Wi-Fi. Available in **GUI**.

![Switch in the FPAS gallery](images/switch.png)

## Example

```pascal
unit Docs.Ui.Selection.Switch;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Inputs.Switches as C;

// Builds the switch example.
// ## Parameters
// - `On`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(On: bool): E.Element<bool>;
begin
  return C.Switch<bool>('wifi', 'Wi-Fi', On, function(Value: bool): bool => Value);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `On` | Pass the Boolean from the model. |
| `Description` | Empty. Adds a second line beneath the label. |
| `Enabled` | true. |

## Interaction

Click anywhere on the row, or press Space or Enter while it has focus. The callback receives the opposite of the displayed value.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/Switches.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Selection/Widgets/Switch.fpas)
- [Checkbox](checkbox.md)
- [Selection](README.md)
