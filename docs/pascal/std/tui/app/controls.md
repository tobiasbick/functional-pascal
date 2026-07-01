# Std.Tui controls

The old retained `Application.HostCreate*View` control API is no longer public.

The current Turbo Vision facade exposes button, static text, input line, list box, and check box handles:

| Symbol | Description |
| --- | --- |
| `Button` | Opaque Turbo Vision button handle. |
| `StaticText` | Opaque non-interactive Turbo Vision text label handle. |
| `InputLine` | Opaque single-line Turbo Vision text input handle. |
| `ListBox` | Opaque Turbo Vision selectable string list handle. |
| `CheckBox` | Opaque Turbo Vision boolean check box handle. |
| `Application.CreateButton(App, Bounds, Text, CommandId): Button` | Create a button. |
| `Application.CreateStaticText(App, Bounds, Text): StaticText` | Create a static text label. |
| `Application.CreateInputLine(App, Bounds, Text, MaxLength): InputLine` | Create a single-line text input with initial text and maximum length. |
| `Application.CreateListBox(App, Bounds, Items, CommandId): ListBox` | Create a selectable list from an array of strings. Enter or double-click dispatches `CommandId` in an interactive Turbo Vision run. |
| `Application.CreateCheckBox(App, Bounds, Text, Checked): CheckBox` | Create a check box with an initial checked state. |
| `Application.AddChild(App, Parent, Child)` | Attach a button, static text, input line, list box, or check box child to a dialog or window. |
| `Application.TestClickButton(App, Button)` | Queue a headless test click for the button. |

Application chrome uses root-level handles:

| Symbol | Description |
| --- | --- |
| `MenuBar` | Opaque Turbo Vision menu bar handle. |
| `MenuBarItem` | Record with `menuText`, `itemText`, and `commandId` fields. |
| `Application.CreateMenuBar(App, Bounds, Items): MenuBar` | Create a menu bar. Each item creates one top-level menu with one command entry. |
| `Application.SetMenuBar(App, MenuBar)` | Set the active application menu bar. |
| `StatusLine` | Opaque Turbo Vision status line handle. |
| `StatusItem` | Record with `text`, `keyCode`, and `commandId` fields. |
| `Application.CreateStatusLine(App, Bounds, Items): StatusLine` | Create a status line. |
| `Application.SetStatusLine(App, StatusLine)` | Set the active application status line. |

## See Also

- [Application](README.md)
- [Native testing](testing.md)
