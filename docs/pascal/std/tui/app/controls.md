# Std.Tui controls

Turbo Vision widget handles and application chrome. Views are created with type-local `*.New` factories, then attached with `Dialog.Add` or `Window.Add`.

## Interactive controls

| Symbol | Description |
| --- | --- |
| `Button.New(Bounds, Text, Command, IsDefault): Button` | Create a button. |
| `Button.SetText(Btn, Text)` | Replace button label text. |
| `StaticText.New(Bounds, Text): StaticText` | Create a static text label. |
| `StaticText.SetText(Txt, Text)` | Replace static text. |
| `Memo.New(Bounds, Text): Memo` | Create a multi-line editor. |
| `Memo.SetText(M, Text)` | Replace memo content. |
| `TextViewer.New(Bounds, Text): TextViewer` | Create a read-only multi-line viewer. |
| `TextViewer.SetText(V, Text)` | Replace viewer content. |
| `InputLine.New(Bounds, Text, MaxLength): InputLine` | Create a single-line input. |
| `InputLine.Text(Line): string` | Read current input text (after `ExecView` or during `Run`). |
| `InputLine.SetText(Line, Text)` | Replace input text. |
| `ListBox.New(Bounds, Items, Command): ListBox` | Create a selectable list. |
| `ListBox.Selection(Lb): integer` | Zero-based selected index, or `-1` when empty. |
| `ListBox.SetItems(Lb, Items)` | Replace list items. |
| `Outline.New(Bounds, Roots): Outline` | Create an outline tree. |
| `Outline.Selection(O): integer` | Zero-based flat visible selection index. |
| `Outline.SelectedText(O): string` | Label of the selected node. |
| `Outline.SetNodes(O, Roots)` | Replace outline nodes. |
| `CheckBox.New(Bounds, Text, Checked): CheckBox` | Create a check box. |
| `CheckBox.Checked(Cb): boolean` | Read checked state. |
| `CheckBox.SetChecked(Cb, Checked)` | Set checked state. |
| `RadioButton.New(Bounds, Text, GroupId, Selected): RadioButton` | Create a radio button. |
| `RadioButton.Selected(Rb): boolean` | Read selected state. |
| `RadioButton.SetSelected(Rb, Selected)` | Set selected state (clears siblings in the group). |

`Dialog.Add(Dlg, Child)` and `Window.Add(Win, Child)` accept `Button`, `StaticText`, `Memo`, `TextViewer`, `InputLine`, `ListBox`, `Outline`, `CheckBox`, and `RadioButton` children.

## Application chrome

| Symbol | Description |
| --- | --- |
| `MenuBar.New(Bounds, Menus): MenuBar` | Create a menu bar. |
| `MenuBar.SetMenus(Bar, Menus)` | Replace menus on a menu bar. |
| `Application.SetMenuBar(App, MenuBar)` | Attach menu bar to the live session. |
| `StatusLine.New(Bounds, Items): StatusLine` | Create a status line. |
| `StatusLine.SetItems(Line, Items)` | Replace status items. |
| `Application.SetStatusLine(App, StatusLine)` | Attach status line to the live session. |

Record types `Menu`, `MenuItem`, `StatusItem`, and `OutlineNode` are defined in [Types](types.md).

## Headless test helpers

| Symbol | Description |
| --- | --- |
| `Application.TestClickButton(App, Button)` | Queue a headless button click (prefer `Test.Click`). |
| `Test.Click(App, Button)` | Preferred headless button click helper. |
| `Application.TestClickMouse(App, X, Y)` | Queue a headless left click at screen coordinates. |
| `Application.TestDispatchMenuCommand(App, MenuBar, MenuIndex, ItemIndex)` | Dispatch a menu item command id (prefer `Test.DispatchMenu`). |
| `Test.DispatchMenu(App, MenuBar, MenuIndex, ItemIndex)` | Preferred headless menu dispatch helper. |

On a live terminal `Application.Run`, the runtime stretches the menu bar and status line to the full terminal width and pins the status line to the bottom row.

## See Also

- [Application](README.md)
- [Dialogs and windows](modals.md)
- [Native testing](testing.md)
- [Types](types.md)
