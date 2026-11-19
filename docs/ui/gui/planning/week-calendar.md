# Week calendar

Use a week calendar to schedule timed events across seven days. The gallery
shows overlapping meetings, calls and blocks of focus time. Available in **GUI**.

![Week calendar in the FPAS gallery](images/week-calendar.png)

## Example

```pascal
unit Docs.Ui.Planning.WeekCalendar;

uses Fpas.Ui.Elements as E, Fpas.Ui.Layout as L, Fpas.Ui.Styles as S,
  Fpas.Ui.Widgets.Planning.WeekCalendars as W, System as Sys;

// Shows the supplied week and appointments.
// ## Parameters
// - `Week`: First day on display.
// - `Appointments`: Model-owned events, in stable order during edits.
// ## Returns
// The calendar with activation and schedule proposals.
public function View(Week: Sys.DateOnly; Appointments: array of W.Appointment): E.Element<W.Event>;
begin
  return W.WeekCalendar<W.Event>('week', new W.Value(Week, Appointments),
    function(Change: W.Event): W.Event => Change,
    new W.Options(StartHour := 8.0F, EndHour := 19.0F,
      Height := L.Extent.Fixed(L.Measure.Logical(420))));
end function;

// Creates an appointment for the first day.
// ## Returns
// A standup from 09:30 to 09:45.
public function Standup(): W.Appointment;
begin
  return new W.Appointment('Standup', 0, 9.5F, 9.75F, 'Zoom', S.Tone.Info);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Appointment.Day / Start / Finish` | Day zero is the first visible day. Express start and finish as fractional hours. |
| `StartHour / EndHour` | 7 / 21. |
| `HourHeight / Height` | 0 / Content. Zero uses native spacing. |
| `ReadOnly / Enabled` | false / true. ReadOnly keeps activation and disables edits. |

## Interaction

Drag an event to move it in 15-minute increments. Drag its bottom edge to
resize. Use the arrows on a focused event, or Shift with Up or Down to
resize. Adopt `Event.Events` to accept an edit. A nonnegative
`Activated` indexes the event to open.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Planning/WeekCalendars.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Planning/View.fpas)
- [Month calendar](month-calendar.md)
- [Planning](README.md)
