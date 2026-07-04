# Std.Tui application

`Std.Tui.Application` is the Turbo Vision application facade for terminal UI code, implemented over the Rust [`turbo-vision`](https://crates.io/crates/turbo-vision) crate from [turbo-vision-4-rust](https://github.com/aovestdipaperino/turbo-vision-4-rust).

## Minimum setup

1. `Application.Open` or `Application.OpenForTest`
2. Create widget handles with `Application.Create*`
3. `Application.AddChild` / `Application.AddWindow` as needed
4. `Application.OnCommand` (and optional `OnKey` / `OnMouse`)
5. `Application.Run` — requires at least one Turbo Vision widget handle

## Current API

| Symbol | Description |
| --- | --- |
| `Application.Open(): Application` | Open a logical TUI application handle. Terminal ownership is acquired by `Application.Run`. |
| `Application.OpenForTest(Width, Height): Application` | Open a headless test application. |
| `Application.Close(App)` | Close an open application handle. |
| `Application.CloseForTest(App)` | Close a headless test application. |
| `Application.Size(App): Size` | Return the current application size. |
| `Application.Run(App)` | Run the Turbo Vision event loop. Requires at least one `Create*` widget handle. |
| `Application.Quit(App)` | Request that `Application.Run` exits. |
| `Application.CreateDialog(App, Bounds, Title): Dialog` | Create a Turbo Vision dialog handle. |
| `Application.CreateWindow(App, Bounds, Title): Window` | Create a Turbo Vision window handle. |
| `Application.CreateButton(App, Bounds, Text, CommandId): Button` | Create a Turbo Vision button handle. |
| `Application.CreateStaticText(App, Bounds, Text): StaticText` | Create a non-interactive Turbo Vision text label handle. |
| `Application.CreateMemo(App, Bounds, Text): Memo` | Create a multi-line Turbo Vision text editor handle. |
| `Application.CreateTextViewer(App, Bounds, Text): TextViewer` | Create a read-only multi-line Turbo Vision text viewer handle. |
| `Application.CreateInputLine(App, Bounds, Text, MaxLength): InputLine` | Create a single-line Turbo Vision text input handle. |
| `Application.CreateListBox(App, Bounds, Items, CommandId): ListBox` | Create a Turbo Vision list box from an array of strings. |
| `Application.CreateCheckBox(App, Bounds, Text, Checked): CheckBox` | Create a Turbo Vision check box with an initial checked state. |
| `Application.CreateRadioButton(App, Bounds, Text, GroupId, Selected): RadioButton` | Create a Turbo Vision radio button. Use the same `GroupId` for mutually exclusive options. |
| `Application.RunFileDialog(App, Bounds, Title, Wildcard, StartPath): Option of string` | Show a modal file dialog. Returns `Some(path)` when a file is chosen and `None` when canceled. Pass `None` as `StartPath` for the current directory. |
| `Application.ExecDialog(App, Dialog): DialogResult` | Run a dialog modally on the terminal. Returns the closing command in `DialogResult.command`. |
| `Application.InputText(App, Field): string` | Read the current text of an `InputLine` handle (valid after `ExecDialog`). |
| `Application.Checked(App, Field): boolean` | Read the checked state of a `CheckBox` handle (valid after `ExecDialog`). |
| `Application.Selected(App, Field): boolean` | Read the selected state of a `RadioButton` handle (valid after `ExecDialog`). |
| `Application.ListSelection(App, ListBox): integer` | Read the zero-based selected list-box index, or `-1` when no item is selected. |
| `Application.AddChild(App, Parent, Child)` | Attach a button, static text, memo, text viewer, input line, list box, check box, or radio button child to a dialog or window parent. |
| `Application.SetText(App, Control, Text)` | Replace the text of a text-bearing control at runtime. |
| `Application.SetChecked(App, Control, Checked)` | Set check box checked state or radio button selected state at runtime. |
| `Application.SetItems(App, ListBox, Items)` | Replace list box items at runtime. |
| `Application.SetTitle(App, Root, Title)` | Replace window or dialog title at runtime. |
| `Application.AddWindow(App, Window)` | Place a window on the application desktop. |
| `Application.CreateMenuBar(App, Bounds, Menus): MenuBar` | Create a top menu bar from an array of `Menu` records. |
| `Application.SetMenuBar(App, MenuBar)` | Set the application menu bar root. |
| `Application.SetMenus(App, MenuBar, Menus)` | Replace menus on an attached menu bar at runtime. |
| `Application.CreateStatusLine(App, Bounds, Items): StatusLine` | Create a bottom status line from an array of `StatusItem` records. |
| `Application.SetStatusLine(App, StatusLine)` | Set the application status line root. |
| `Application.SetStatusItems(App, StatusLine, Items)` | Replace items on an attached status line at runtime. |
| `Application.OnCommand(App, Handler)` | Register `procedure (Application, integer)` for command dispatch. Use `Command.*` constants for standard actions. |
| `Application.OnKey(App, Handler)` | Optional hook: `function (Application, Std.Console.KeyEvent): boolean` for keys left unhandled by the widget tree. |
| `Application.OnMouse(App, Handler)` | Optional hook: `procedure (Application, Std.Console.Event)` for mouse events left unhandled by the widget tree. |
| `Application.Pump(App): integer` | Process one headless Turbo Vision pump step. |
| `Application.TestClickButton(App, Button)` | Queue a test click for a Turbo Vision button. |
| `Application.TestClickMouse(App, X, Y)` | Queue a headless left click at screen coordinates on a check box or radio button. |
| `Application.TestDispatchMenuCommand(App, MenuBar, MenuIndex, ItemIndex)` | Queue a menu item command for headless tests. |
| `Application.TestSetFileDialogResult(App, Result)` | Queue the `Option of string` returned by the next headless `Application.RunFileDialog` call. |
| `Application.TestSetDialogResult(App, Command)` | Queue the closing command returned by the next headless `Application.ExecDialog` call. |

For headless screen assertions after `Pump`, use [`Std.Test`](../../testing/test.md) `AssertScreenLine` / `AssertScreenCell` with `uses Std.Console`.

## See Also

- [Session API](../session.md)
- [Application types](types.md)
- [Controls](controls.md)
- [Dialogs and windows](modals.md)
- [File dialog](file-dialog.md)
- [Handlers](handlers.md)
- [Lifecycle](lifecycle.md)
- [Native testing](testing.md)
- [VM bridge](vm-bridge.md)
