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

Use `DateOnly` for calendar dates. The week calendar takes fractional hours.
A Gantt chart takes day offsets relative to its origin date.

## See also

- [UI handbook](../README.md)
- [Date picker](../inputs/date-picker.md)
- [Planning in the gallery](../../../../examples/ui/gui/gallery/Pages/Planning/View.fpas)

## Source files

- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Planning/State.fpas)
- [CuiWindow.ScheduleParts.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Planning/CuiWindow.ScheduleParts.cs)
- [planning.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/planning.c)

The [FPAS interaction contract](../../../../tests/Fixtures/gui-host/PlanningInteractionContract.fpas) exercises leap-day navigation, interrupted appointment drags, read-only Gantt activation, and tree expansion after reordering.
