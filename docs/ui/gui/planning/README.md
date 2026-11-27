# Planning

The gallery shows a task board, two calendars and a Gantt chart. The application owns
card order, appointments and task dates. Each widget renders that state and
reports proposed changes. These widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Kanban](kanban.md) | Move cards within and between columns. |
| [Week calendar](week-calendar.md) | Show timed appointments and let users move or resize them. |
| [Gantt chart](gantt.md) | Show task dates, progress, milestones and dependencies. |
| [Month calendar](month-calendar.md) | Browse dates and show timed, all-day and multi-day appointments. |

## Planning gallery page

The Kanban board has four columns that scroll horizontally. The second
column shows a work-in-progress limit of three. Each card has a badge,
caption and assignee. Cards show progress, attachments and comments when present.

The week calendar, Gantt chart and month calendar fill the page width.
The week calendar covers 08:00 to 19:00. The Gantt chart covers 42 days.
The month calendar shows appointments for the current month and preserves
the user's displayed month and selection.

## Dates

Use `DateOnly` for calendar dates. Use `DateOnly.DayNumber` and
`DateOnly.FromDayNumber` to convert between dates and day numbers.
The GUI adapter converts between the CLR and CUI date domains.

The gallery reads the local calendar day with `DateTime.Today`. It uses
`DayOfWeek` and `AddDays` to find Monday and the Gantt origin.
The week calendar takes fractional hours. The Gantt chart takes offsets
in days from its origin date.

## See also

- [UI handbook](../README.md)
- [Date picker](../inputs/date-picker.md)
- [Planning in the gallery](../../../../examples/ui/gui/gallery/Pages/Planning/View.fpas)
- [CUI gallery comparison](../../../future/cui-compare/README.md#planning-evidence)

## Source files

- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Planning/State.fpas)
- [CuiWindow.ScheduleParts.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Planning/CuiWindow.ScheduleParts.cs)
- [planning.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/planning.c)

The [FPAS interaction contract](../../../../tests/Fixtures/gui-host/PlanningInteractionContract.fpas) exercises leap-day navigation, interrupted appointment drags, read-only Gantt activation, and tree expansion after reordering.
