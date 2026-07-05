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

Standard command identifiers for buttons and `OnCommand` handlers.
Application-defined commands use other positive integers. When a command id collides with an upstream Turbo Vision `CM_*` id, the runtime offsets it before handing it to Turbo Vision and restores the original value in `OnCommand`. This includes upstream standard, broadcast, file/edit/search/view/help, and demo command ids from `turbo-vision` 2.0. The four `Command.*` constants pass through unchanged because they match Borland values.

| Constant | Value | Meaning |
| --- | --- | --- |
| `Command.Quit` | `1` | Exit the application (`Application.Quit`; Borland `cmQuit`). |
| `Command.Close` | `4` | Close the source view or window (Borland `cmClose`). |
| `Command.Accept` | `10` | Accept or confirm (dialog OK; Borland `cmOK`). Named `Accept` because `Ok` is a language keyword. |
| `Command.Cancel` | `11` | Cancel the current action or dialog (Borland `cmCancel`). |

Applications may intentionally reuse other Borland `CM_*` values for menu items when they match upstream semantics. The FPAS IDE Help → About entry uses `100` (`CM_ABOUT` in turbo-vision 2.0). The runtime offsets that id on Turbo Vision widgets and restores `100` in `OnCommand`. Standard message-box OK buttons use `Command.Accept` (`10`, Borland `cmOK`) — see the IDE About flow in [Dialogs and windows](modals.md).

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

## `Menu`

Record used by `Application.CreateMenuBar` for one top-level menu.

| Field | Type | Meaning |
| --- | --- | --- |
| `title` | `string` | Top-level menu text. Use `~X~` markers for accelerators. |
| `items` | `array of MenuItem` | Entries inside the menu. |

## `MenuItem`

Record for one menu entry or separator.

| Field | Type | Meaning |
| --- | --- | --- |
| `text` | `string` | Item label. Ignored when `commandId` is `0`. |
| `commandId` | `integer` | Command dispatched when selected. Use `0` for a separator line. |

## `StatusLine`

Opaque Turbo Vision status line handle returned by `Application.CreateStatusLine`.

## `StatusItem`

Record used by `Application.CreateStatusLine`.

| Field | Type | Meaning |
| --- | --- | --- |
| `text` | `string` | Status item text. Use `~X~` markers for highlighted shortcuts. |
| `keyCode` | `integer` | Turbo Vision key code. Use `0` when no keyboard shortcut is attached. |
| `commandId` | `integer` | Command id dispatched by the status item. |

## See Also

- [Application](README.md)
- [Dialogs and windows](modals.md) — `Command.Accept`, menu `CM_ABOUT`, custom vs standard modals
- [Session API](../session.md)
