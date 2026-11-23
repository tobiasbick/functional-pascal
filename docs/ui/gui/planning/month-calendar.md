# Month calendar

Use a month calendar to browse appointments, holidays and days off. The
gallery shows timed events in order, bars for multi-day events and a
count of additional appointments when a day is full. Available in **GUI**.

![Month calendar in the FPAS gallery](images/month-calendar.png)

## Example

```pascal
unit Docs.Ui.Planning.MonthCalendar;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Planning.MonthCalendars as W,
  System as Sys;

// Shows the current month and its appointments.
// ## Parameters
// - `Current`: Displayed month, selected date and appointment array.
// ## Returns
// The calendar with navigation and activation proposals.
public function View(Current: W.Value): E.Element<W.Event>;
begin
  return W.MonthCalendar<W.Event>('release-calendar', Current,
    function(Change: W.Event): W.Event => Change);
end function;

// Creates a three-day appointment in March.
// ## Returns
// The appointment, with an inclusive final date.
public function Offsite(): W.Appointment;
begin
  return new W.Appointment('Offsite', new Sys.DateOnly(2026, 3, 9),
    new Sys.DateOnly(2026, 3, 11), AllDay := true);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Value.Month / Selected` | Supply a date in the displayed month. The day is ignored. Selection defaults to null. |
| `Appointment.Finish / Time / AllDay` | null / 9 / false. A finish date is inclusive and Time uses fractional hours. |
| `FirstWeekday / CellHeight` | 0 / 0. Monday is zero and zero height uses native sizing. |
| `MonthNames / DayNames` | Empty. For custom names, supply twelve months and seven weekdays starting with Monday. |
| `Hour12 / Today / Enabled` | false / null / true. Null Today uses the current date. |

## Interaction

Adopt the proposed month and selected date in your update. Use
`MonthChanged` to load appointments for a different month and `DayActivated`
to open the selected day. A nonnegative `Activated` indexes an appointment
in the array supplied to the widget.

With the grid focused, Page Up and Page Down keep the day of the month when
possible and select the last available day in shorter months. Space selects
the keyboard-focused date and Enter activates it, even when the pointer rests
over a different date. Clicking selects the date under the pointer.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Planning/MonthCalendars.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Planning/View.fpas)
- [Week calendar](week-calendar.md)
- [Planning](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Planning/`
  - [MonthCalendar.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Planning/MonthCalendar.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Planning/`
  - [NativeMethods.MonthCalendar.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Planning/NativeMethods.MonthCalendar.Types.cs)
  - [NativeMethods.MonthCalendar.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Planning/NativeMethods.MonthCalendar.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Planning/`
  - [CuiWindow.MonthCalendar.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Planning/CuiWindow.MonthCalendar.Types.cs)
  - [CuiWindow.MonthCalendar.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Planning/CuiWindow.MonthCalendar.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/planning/`
  - [month_calendar.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/planning/month_calendar.c)
  - [month_calendar.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/planning/month_calendar.h)
