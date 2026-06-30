# Std.Tui dialogs

The retained modal stack and `ShowModal` APIs are removed.

The current Turbo Vision spike exposes dialog handles:

| Symbol | Description |
| --- | --- |
| `Dialog` | Opaque Turbo Vision dialog handle. |
| `Application.CreateDialog(App, Bounds, Title): Dialog` | Create a dialog. |
| `Application.AddChild(App, Dialog, Button)` | Attach a button child to a dialog. |

## See Also

- [Application](README.md)
- [Controls](controls.md)
