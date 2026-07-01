# Std.Tui dialogs

The retained modal stack and `ShowModal` APIs are removed.

The current Turbo Vision spike exposes dialog handles:

| Symbol | Description |
| --- | --- |
| `Dialog` | Opaque Turbo Vision dialog handle. |
| `Application.CreateDialog(App, Bounds, Title): Dialog` | Create a dialog. |
| `Application.AddChild(App, Dialog, Child)` | Attach a button, static text, input line, list box, or check box child to a dialog. |

## See Also

- [Application](README.md)
- [Controls](controls.md)
