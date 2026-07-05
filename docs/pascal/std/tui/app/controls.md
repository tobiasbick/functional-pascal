# Std.Tui controls

Turbo Vision widget handles and application chrome.

| Symbol | Description |
| --- | --- |
| `Button` | Opaque Turbo Vision button handle. |
| `StaticText` | Opaque non-interactive Turbo Vision text label handle. |
| `Memo` | Opaque multi-line Turbo Vision text editor handle. |
| `TextViewer` | Opaque read-only multi-line Turbo Vision text viewer handle. |
| `InputLine` | Opaque single-line Turbo Vision text input handle. |
| `ListBox` | Opaque Turbo Vision selectable string list handle. |
| `CheckBox` | Opaque Turbo Vision boolean check box handle. |
| `RadioButton` | Opaque Turbo Vision mutually exclusive radio button handle. |
| `Application.CreateButton(App, Bounds, Text, CommandId): Button` | Create a button. |
| `Application.CreateStaticText(App, Bounds, Text): StaticText` | Create a static text label. |
| `Application.CreateMemo(App, Bounds, Text): Memo` | Create a multi-line text editor with initial content. |
| `Application.CreateTextViewer(App, Bounds, Text): TextViewer` | Create a read-only multi-line text viewer with initial content. |
| `Application.CreateInputLine(App, Bounds, Text, MaxLength): InputLine` | Create a single-line text input with initial text and maximum length. |
| `Application.CreateListBox(App, Bounds, Items, CommandId): ListBox` | Create a selectable list from an array of strings. Enter or double-click dispatches `CommandId` in an interactive Turbo Vision run. |
| `Application.CreateCheckBox(App, Bounds, Text, Checked): CheckBox` | Create a check box with an initial checked state. Left-click toggles during `Application.Run`. |
| `Application.CreateRadioButton(App, Bounds, Text, GroupId, Selected): RadioButton` | Create a radio button in a group. Buttons with the same `GroupId` are mutually exclusive. Left-click selects during `Application.Run`. |
| `Application.AddChild(App, Parent, Child)` | Attach a button, static text, memo, text viewer, input line, list box, check box, or radio button child to a dialog or window. |
| `Application.SetText(App, Control, Text)` | Replace the text of a button, static text, memo, text viewer, input line, check box, or radio button. The change re-renders live during `Application.Run` and is visible to headless screen queries. Not supported for list boxes. |
| `Application.SetChecked(App, Control, Checked)` | Set the checked state of a check box or the selected state of a radio button. Selecting a radio button clears other buttons in the same group. |
| `Application.Selected(App, RadioButton): boolean` | Read a radio button's selected state. Use after `Application.ExecDialog` for modal choices. |
| `Application.ListSelection(App, ListBox): integer` | Read the zero-based selected list-box index. Returns `-1` when the list is empty. |
| `Application.SetItems(App, ListBox, Items)` | Replace the string items of a list box. |
| `Application.SetTitle(App, Root, Title)` | Replace the title of a window or dialog root. |
| `Application.TestClickButton(App, Button)` | Queue a headless test click for the button. |
| `Application.TestClickMouse(App, X, Y)` | Queue a headless left click at screen coordinates on a check box or radio button. |

Application chrome uses root-level handles:

| Symbol | Description |
| --- | --- |
| `MenuBar` | Opaque Turbo Vision menu bar handle. |
| `Menu` | Record with `title` and `items` for one top-level menu. |
| `MenuItem` | Record with `text` and `commandId` (`0` = separator). |
| `Application.CreateMenuBar(App, Bounds, Menus): MenuBar` | Create a menu bar from an array of `Menu` records. |
| `Application.SetMenuBar(App, MenuBar)` | Set the active application menu bar. |
| `Application.SetMenus(App, MenuBar, Menus)` | Replace the menus on an attached menu bar at runtime. Re-renders live and in headless queries. |
| `StatusLine` | Opaque Turbo Vision status line handle. |
| `StatusItem` | Record with `text`, `keyCode`, and `commandId` fields. |
| `Application.CreateStatusLine(App, Bounds, Items): StatusLine` | Create a status line. |
| `Application.SetStatusLine(App, StatusLine)` | Set the active application status line. |
| `Application.SetStatusItems(App, StatusLine, Items)` | Replace the items on an attached status line at runtime. Re-renders live and in headless queries. |

On a live terminal `Application.Run`, the runtime stretches the menu bar and status line to the full terminal width and pins the status line to the bottom row (matching Turbo Vision resize behavior). Headless `OpenForTest` runs keep the `Bounds` you pass to `Create*`.

`TextViewer` and other controls are composed into custom dialogs (for example the FPAS IDE About box today). Standard Borland message boxes do not use these widgets — see [Dialogs and windows](modals.md#custom-modal-layout).

## See Also

- [Application](README.md)
- [Dialogs and windows](modals.md)
- [Native testing](testing.md)
