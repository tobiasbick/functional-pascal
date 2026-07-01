# Std.Tui dialogs and windows

Turbo Vision dialogs and windows are opaque handles created before `Application.Run`.

| Symbol | Description |
| --- | --- |
| `Dialog` | Modal or modeless dialog handle. |
| `Window` | Desktop window handle. |
| `Application.CreateDialog(App, Bounds, Title): Dialog` | Create a dialog. |
| `Application.CreateWindow(App, Bounds, Title): Window` | Create a window. |
| `Application.AddWindow(App, Window)` | Place a window on the application desktop. |
| `Application.AddChild(App, Parent, Child)` | Attach a control child to a dialog or window parent. |

`Parent` may be a `Dialog` or `Window`. `Child` may be a `Button`, `StaticText`, `Memo`, `InputLine`, `ListBox`, `CheckBox`, or `RadioButton`.

The retained modal stack and `ShowModal` APIs are removed.

## See Also

- [Application](README.md)
- [Controls](controls.md)
- [Types](types.md)
