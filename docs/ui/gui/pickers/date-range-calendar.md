# Date range calendar

Use an inline range calendar when both date choices should stay visible. The gallery shares the range with the Stay dates field. Available in **GUI**.

![Date range calendar in the FPAS gallery](images/date-range-calendar.png)

## Example

```pascal
unit Docs.Ui.Pickers.DateRangeCalendar;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Pickers.DateRangeCalendars as W,
  Fpas.Ui.Widgets.Pickers.DateRanges as R;

// Builds the date range calendar example.
// ## Parameters
// - `Stay`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Stay: R.Value): E.Element<R.Value>;
begin
  return W.DateRangeCalendar<R.Value>('picker-inline-range', Stay, function(Value: R.Value): R.Value => Value);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Value` | An empty, started or complete DateRanges.Value. |
| `Months` | 2. Set to 1 for a single month. |
| `Enabled` | true. |

## Interaction

Choose a day for the first endpoint, then choose the second. The callback receives both partial and complete proposals. Store either value in the model to keep the inline calendar and picker in sync. Each calendar keeps its own displayed month, and the navigation arrows can move across year boundaries.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Pickers/DateRangeCalendars.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Pickers/Widgets/DateRange.fpas)
- [Date range picker](date-range-picker.md)
- [Pickers](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Pickers/`
  - [DateRangeCalendar.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Pickers/DateRangeCalendar.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Pickers/`
  - [NativeMethods.DateRange.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Pickers/NativeMethods.DateRange.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Pickers/`
  - [CuiWindow.DateRange.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Pickers/CuiWindow.DateRange.Types.cs)
  - [CuiWindow.DateRange.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Pickers/CuiWindow.DateRange.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/pickers/`
  - [date_range_calendar.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/date_range_calendar.c)
  - [date_range_calendar.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/date_range_calendar.h)
