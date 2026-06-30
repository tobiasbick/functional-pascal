# Std.Tui controls

The old retained `Application.HostCreate*View` control API is no longer public.

The current Turbo Vision spike exposes only button handles:

| Symbol | Description |
| --- | --- |
| `TuiButton` | Opaque Turbo Vision button handle. |
| `Application.CreateButton(App, Bounds, Text, CommandId): TuiButton` | Create a button. |
| `Application.AddChild(App, Dialog, Button)` | Attach the button to a dialog. |
| `Application.TestClickButton(App, Button)` | Queue a headless test click for the button. |

## See Also

- [Application](README.md)
- [Native testing](testing.md)
