# Time dial

Use an analog dial to select hours and minutes directly on a clock face. Available in **GUI**.

![Time dial in the FPAS gallery](images/time-dial.png)

## Example

```pascal
unit Docs.Ui.Pickers.TimeDial;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Pickers.TimeDials as W;
uses System as Sys;

// Builds the time dial example.
// ## Parameters
// - `Time`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Time: Sys.TimeOnly): E.Element of Sys.TimeOnly;
begin
  return W.TimeDial of Sys.TimeOnly('picker-time-dial', Time, function(Value: Sys.TimeOnly): Sys.TimeOnly => Value,
    Size := 232.0F);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | Stable dial ID. |
| `Value` | System.TimeOnly at minute precision. |
| `Size` | Finite face diameter in logical units. Nonpositive uses 248. Positive values below 160 use 160. |
| `Enabled` | true. |

## Interaction

Activate the hour or minute selector and drag the clock hand. Releasing the hour hand switches the face to minutes. The outer ring shows 1 through 12. The inner ring shows 00 and 13 through 23. Use `SelectorId` and `FaceId` to identify the native controls.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_time_dial)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Pickers/TimeDials.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Pickers/Widgets/WheelsAndDial.fpas)
- [Time picker](time-picker.md)
- [Pickers](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Pickers/`
  - [TimeDial.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Pickers/TimeDial.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Pickers/`
  - [NativeMethods.TimeDial.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Pickers/NativeMethods.TimeDial.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Pickers/`
  - [CuiWindow.TimeDial.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Pickers/CuiWindow.TimeDial.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/pickers/`
  - [time_dial.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/time_dial.c)
  - [time_dial.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/time_dial.h)
