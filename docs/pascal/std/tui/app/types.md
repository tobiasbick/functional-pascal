# Std.Tui types

This page defines the complete public FPAS type surface of `Std.Tui`.

## `Application`

Opaque handle for a TUI application session.

## `TuiRect`

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

## Command ids (`CM_*`)

Command ids are plain integers. Upstream Turbo Vision `CM_*` constants are exported from `Std.Tui` and pass through to `OnCommand` unchanged.

| Constant | Value | Meaning |
| --- | --- | --- |
| `CM_QUIT` | `1` | Exit the application (`Application.Quit`; Borland `cmQuit`). |
| `CM_CLOSE` | `4` | Close the source view or window (Borland `cmClose`). |
| `CM_OK` | `10` | Accept or confirm (dialog OK; Borland `cmOK`). |
| `CM_CANCEL` | `11` | Cancel the current action or dialog (Borland `cmCancel`). |
| `CM_ABOUT` | `100` | Standard About command (IDE Help → About). |
| `CM_OPEN` | `102` | Standard Open command (IDE File → Open). |
| `CM_USER` | `4096` | Suggested base for application-private command ids. |

Application-defined commands use other positive integers. Avoid reusing upstream reserved `CM_*` ids for custom widgets when possible.

## `MessageBoxOption` constants

Option flags for [`Application.MessageBox`](message-box.md). Combine type and button flags with `+`.

| Constant | Value |
| --- | --- |
| `MessageBoxOption.Warning` | `0` |
| `MessageBoxOption.Error` | `1` |
| `MessageBoxOption.Information` | `2` |
| `MessageBoxOption.Confirmation` | `3` |
| `MessageBoxOption.About` | `4` |
| `MessageBoxOption.YesButton` | `256` |
| `MessageBoxOption.NoButton` | `512` |
| `MessageBoxOption.OkButton` | `1024` |
| `MessageBoxOption.CancelButton` | `2048` |
| `MessageBoxOption.OkCancel` | `3072` |
| `MessageBoxOption.YesNoCancel` | `3840` |

## `Window`

Opaque Turbo Vision window handle from `Window.New`. Call `Desktop.Add` before `Application.Run` to show the window.

## `Dialog`

Opaque Turbo Vision dialog handle from `Dialog.NewModal`.

## `Button`

Opaque button handle from `Button.New`.

## `StaticText`

Opaque static text handle from `StaticText.New`.

## `Memo`

Opaque multi-line editor handle from `Memo.New`.

## `TextViewer`

Opaque read-only multi-line viewer handle from `TextViewer.New`.

## `InputLine`

Opaque single-line input handle from `InputLine.New`.

## `ListBox`

Opaque list box handle from `ListBox.New`.

## `Outline`

Opaque outline handle from `Outline.New`. Backed by upstream `OutlineViewer`.

## `OutlineNode`

Record describing one node in an outline tree.

| Field | Type | Meaning |
| --- | --- | --- |
| `text` | `string` | Label shown for the node. |
| `children` | `array of OutlineNode` | Child nodes. Use an empty array for leaves. |
| `expanded` | `boolean` | When `true`, child nodes are visible in the outline. |

## `CheckBox`

Opaque check box handle from `CheckBox.New`.

## `RadioButton`

Opaque radio button handle from `RadioButton.New`. Buttons that share a `GroupId` form one mutually exclusive group.

## `MenuBar`

Opaque menu bar handle from `MenuBar.New`.

## `Menu`

Record used by `MenuBar.New` for one top-level menu.

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

Opaque status line handle from `StatusLine.New`.

## `StatusItem`

Record used by `StatusLine.New`.

| Field | Type | Meaning |
| --- | --- | --- |
| `text` | `string` | Status item text. Use `~X~` markers for highlighted shortcuts. |
| `keyCode` | `integer` | Turbo Vision key code. Use `0` when no keyboard shortcut is attached. |
| `commandId` | `integer` | Command id dispatched by the status item. |

## See Also

- [Application](README.md)
- [Dialogs and windows](modals.md)
- [Session API](../session.md)
