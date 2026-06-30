# Terminal UI

Terminal UI APIs for Functional Pascal.

The current `Std.Tui` surface is a small application facade plus a Turbo Vision command-callback spike. The previous retained public host API has been removed from the Pascal surface.

| Topic | Description |
| --- | --- |
| [Session API](session.md) | `Application.Open`, `Run`, `Close`, size, redraw |
| [Application](app/README.md) | Current application, Turbo Vision handles, and test helpers |
| [Types](app/types.md) | `Application`, `Rect`, `TuiDialog`, `TuiButton`, transition query records |
| [Native testing](app/testing.md) | Headless tests with `OpenForTest` and `Test*` helpers |
| [Terminal checklist](terminal-checklist.md) | Local verification commands |
| [Cell width](cell-width.md) | Unicode display-width policy |

## See Also

- [`Std.Console`](../console/README.md)
- [Std index](../README.md)
