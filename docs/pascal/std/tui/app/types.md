# Std.Tui types

## `Application`

Opaque handle for a TUI application session.

## `Rect`

Rectangle in terminal cells.

| Field | Type | Meaning |
| --- | --- | --- |
| `x` | `integer` | Left edge. |
| `y` | `integer` | Top edge. |
| `width` | `integer` | Width in cells. |
| `height` | `integer` | Height in cells. |

## `Point`

| Field | Type | Meaning |
| --- | --- | --- |
| `x` | `integer` | Horizontal cell offset. |
| `y` | `integer` | Vertical cell offset. |

## `Size`

| Field | Type | Meaning |
| --- | --- | --- |
| `width` | `integer` | Width in cells. |
| `height` | `integer` | Height in cells. |

## `Command` constants

Standard command identifiers for buttons and `OnCommand` handlers. Application-defined commands use other positive integers.

| Constant | Value | Meaning |
| --- | --- | --- |
| `Command.Accept` | `1` | Accept or confirm (dialog OK). Named `Accept` because `Ok` is a language keyword. |
| `Command.Cancel` | `2` | Cancel the current action or dialog. |
| `Command.Close` | `3` | Close the source view or window. |
| `Command.Quit` | `4` | Exit the application (`Application.Quit`). |

## `Window`

Opaque Turbo Vision window handle returned by `Application.CreateWindow`. Call `Application.AddWindow` before `Application.Run` to show the window on the desktop.

## `Dialog`

Opaque Turbo Vision dialog handle returned by `Application.CreateDialog`.

## `DialogResult`

Result of `Application.ExecDialog`.

| Field | Type | Meaning |
| --- | --- | --- |
| `command` | `integer` | The command id that closed the dialog (for example `Command.Accept`). |

## `Button`

Opaque Turbo Vision button handle returned by `Application.CreateButton`.

## `StaticText`

Opaque non-interactive Turbo Vision text label handle returned by `Application.CreateStaticText`.

## `Memo`

Opaque multi-line Turbo Vision text editor handle returned by `Application.CreateMemo`. Initial `Text` may contain newline characters.

## `TextViewer`

Opaque read-only multi-line Turbo Vision text viewer handle returned by `Application.CreateTextViewer`. Initial `Text` may contain newline characters. Use for logs, help text, and long read-only content. Distinct from `Memo` (editable) and `StaticText` (short label).

## `InputLine`

Opaque single-line Turbo Vision text input handle returned by `Application.CreateInputLine`.

## `ListBox`

Opaque Turbo Vision selectable string list handle returned by `Application.CreateListBox`.

## `CheckBox`

Opaque Turbo Vision boolean check box handle returned by `Application.CreateCheckBox`.

## `RadioButton`

Opaque Turbo Vision radio button handle returned by `Application.CreateRadioButton`. Buttons that share a `GroupId` form one mutually exclusive group.

## `MenuBar`

Opaque Turbo Vision menu bar handle returned by `Application.CreateMenuBar`.

## `MenuBarItem`

Record used by `Application.CreateMenuBar`.

| Field | Type | Meaning |
| --- | --- | --- |
| `menuText` | `string` | Top-level menu text. Use `~X~` markers for accelerators. |
| `itemText` | `string` | Command entry text inside that top-level menu. |
| `commandId` | `integer` | Command id dispatched by the menu item. |

## `StatusLine`

Opaque Turbo Vision status line handle returned by `Application.CreateStatusLine`.

## `StatusItem`

Record used by `Application.CreateStatusLine`.

| Field | Type | Meaning |
| --- | --- | --- |
| `text` | `string` | Status item text. Use `~X~` markers for highlighted shortcuts. |
| `keyCode` | `integer` | Turbo Vision key code. Use `0` when no keyboard shortcut is attached. |
| `commandId` | `integer` | Command id dispatched by the status item. |

## `ScreenCell`

| Field | Type | Meaning |
| --- | --- | --- |
| `ch` | `string` | Cell text. |
| `fg` | `integer` | Foreground color. |
| `bg` | `integer` | Background color. |

## `ApplicationHandlers`

Record for bundled transition handlers used by `Application.Configure`. Optional handler fields use `Some(Handler)` or `None`.

For new Turbo Vision command dispatch, prefer `Application.OnCommand(App, Handler)`.

## `ExitReason`

Registered enum for transition run-loop exit reporting.

| Variant | Meaning |
| --- | --- |
| `UserQuit` | The application requested exit. |
| `HostStop` | The backend stopped the active run. |
| `HostAndUserStop` | Both stop paths happened during the same run. |
| `HostShutdown` | The VM entered shutdown while the run was active. |

## See Also

- [Application](README.md)
- [Session API](../session.md)
