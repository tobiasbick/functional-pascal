# Terminal UI

Terminal UI APIs for Functional Pascal.

`Std.Tui` exposes a Turbo Vision application facade backed by the Rust `turbo-vision` crate. Widgets are opaque handles created before `Application.Run` and attached with `Application.AddChild`, `Application.AddWindow`, `Application.SetMenuBar`, or `Application.SetStatusLine`. Command dispatch uses `Application.OnCommand` and `Command.*` constants.

A legacy hosted global-handler loop (`Application.Configure` with `OnPaint`, `OnKeyPressed`, and related handlers) remains for transition apps that do not construct Turbo Vision widgets.

| Topic | Description |
| --- | --- |
| [Session API](session.md) | `Application.Open`, `Run`, `Close`, size, redraw |
| [Application](app/README.md) | Full `Application.*` reference |
| [Types](app/types.md) | Handles, `Rect`, `Command` constants, menu/status records |
| [Controls](app/controls.md) | Buttons, text fields, lists, check boxes, radio buttons, menu bar, status line |
| [Dialogs and windows](app/modals.md) | `Dialog`, `Window`, and child attachment |
| [File dialog](app/file-dialog.md) | Modal `Application.RunFileDialog` |
| [Handlers](app/handlers.md) | `ApplicationHandlers`, `Application.OnCommand` |
| [Lifecycle](app/lifecycle.md) | Open, run, quit, and close rules |
| [Native testing](app/testing.md) | Headless tests with `OpenForTest` and `Test*` helpers |
| [VM bridge](app/vm-bridge.md) | Pascal-to-intrinsic map for contributors |
| [Terminal checklist](terminal-checklist.md) | Local verification commands |
| [Cell width](cell-width.md) | Unicode display-width policy |

## See Also

- [`Std.Console`](../console/README.md)
- [Std index](../README.md)
