# Std.Tui dialogs

The current Turbo Vision spike exposes dialog handles:

| Symbol | Description |
| --- | --- |
| `TuiDialog` | Opaque Turbo Vision dialog handle. |
| `Application.CreateDialog(App, Bounds, Title): TuiDialog` | Create a dialog. |
| `Application.AddChild(App, Dialog, Button)` | Attach a button child to a dialog. |

The older retained modal helpers are transition internals unless they are listed in [Application](README.md).

## See Also

- [Application](README.md)
- [Controls](controls.md)
