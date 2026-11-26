# Calendar

Use a calendar to leave single-date selection visible. The gallery shares its selection with the Due date field. Available in **GUI**.

![Calendar in the FPAS gallery](images/calendar.png)

## Example

```pascal
unit Docs.Ui.Inputs.Calendar;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Pickers.Calendars as W;
uses System as Sys;

// Builds the calendar example.
// ## Parameters
// - `Due`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Due: Sys.DateOnly?): E.Element of Sys.DateOnly;
begin
  return W.Calendar of Sys.DateOnly('input-calendar', Due, function(Value: Sys.DateOnly): Sys.DateOnly => Value);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Value` | A System.DateOnly, or null for no selection. |
| `Enabled` | true. |

## Interaction

Choose a day to propose a date. The title switches from days to twelve months and then to twelve years. Choosing a year returns to the months. Choosing a month returns to days without changing the selected date. The calendar keeps its own displayed month. Use `TitleId`, `MonthId` and `DayId` to address visible controls.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_calendar)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Pickers/Calendars.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Inputs/Widgets/InlineCalendar.fpas)
- [Date picker](date-picker.md)
- [Inputs](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Pickers/`
  - [Calendar.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Pickers/Calendar.fpas)
  - [CalendarParts.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Pickers/CalendarParts.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Pickers/`
  - [NativeMethods.Calendar.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Pickers/NativeMethods.Calendar.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Pickers/`
  - [CuiWindow.Calendar.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Pickers/CuiWindow.Calendar.Types.cs)
  - [CuiWindow.Calendar.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Pickers/CuiWindow.Calendar.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [calendar.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/calendar.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/pickers/`
  - [calendar.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/calendar.c)
  - [calendar.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/calendar.h)
  - [calendar_parts.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/calendar_parts.c)
- `tests/Fpas.Ui.Gui.Cui.Tests/Inputs/`
  - [CuiCalendarTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Inputs/CuiCalendarTests.cs)
