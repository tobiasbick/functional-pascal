# Time picker

Use a time picker for a clock time in a form. The gallery has a 24-hour field and a 12-hour field with quarter-hour choices. Available in **GUI**.

![24-hour and 12-hour time fields in the FPAS gallery](images/time-picker-fields.png)

![12-hour time picker open in the FPAS gallery](images/time-picker.png)

## Example

```pascal
unit Docs.Ui.Pickers.TimePicker;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Pickers.TimePickers as W;
uses System as Sys;

// Builds the time picker example.
// ## Parameters
// - `Time`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Time: Sys.TimeOnly): E.Element of Sys.TimeOnly;
begin
  return W.TimePicker of Sys.TimeOnly('picker-end-time', Time, function(Value: Sys.TimeOnly): Sys.TimeOnly => Value,
    new W.Options(TwelveHour := true, MinuteStep := 15));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | Nonempty stable control ID. |
| `Value` | `System.TimeOnly` at minute precision. |
| `OnChange` | Maps a proposed time to a message. |
| `Label` | Null omits the label wrapper. An empty string keeps the wrapper without a caption. |
| `Width` | `Layout.Extent.Content`. A labeled field fills the available width. An unlabeled field prefers 150 logical units. Set Fill, Fixed or Percent to choose a width explicitly. |
| `TwelveHour` | `false`. Set to `true` for a 12-hour display and an AM/PM column. |
| `MinuteStep` | 5. Use 1 through 60 to space the minute choices. |
| `Enabled` | `true`. Setting `false` closes the popup and releases focus. |

## Interaction

Click the field or press Enter while it has focus. Select an hour, a minute
or a period in the open popup. A different choice proposes a new time.
The popup stays open. Scroll a column to reach more choices. Escape closes
the popup and returns focus to the field.

`MinuteStep` controls the choices in the popup. The field retains the model's
exact minute until the user chooses a new one. In 12-hour mode, choosing AM
or PM changes the period and keeps the hour and minute.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_time_picker)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Pickers/TimePickers.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Pickers/Widgets/Time.fpas)
- [Time dial](time-dial.md)
- [Pickers](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Pickers/`
  - [TimePicker.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Pickers/TimePicker.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Pickers/`
  - [NativeMethods.TimePicker.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Pickers/NativeMethods.TimePicker.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Pickers/`
  - [CuiWindow.TimePicker.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Pickers/CuiWindow.TimePicker.Types.cs)
  - [CuiWindow.TimePicker.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Pickers/CuiWindow.TimePicker.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/pickers/`
  - [time_picker.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/time_picker.c)
  - [time_picker.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/time_picker.h)
- `tests/Fixtures/gui-host/`
  - [TimePickersContract.fpas](../../../../tests/Fixtures/gui-host/TimePickersContract.fpas)
- `tests/Fpas.Ui.Gui.Cui.Tests/Inputs/`
  - [CuiTimePickerTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Inputs/CuiTimePickerTests.cs)
