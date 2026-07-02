# Std.Tui application

`Std.Tui.Application` is the application facade for terminal UI code.

The public surface is implemented over the Rust `turbo-vision` crate. The old retained `Application.Host*` API is not registered. Calls to removed retained APIs such as `Application.Host*`, retained view queries, modal queries, and `Application.ShowFramedDialog` report a Sema error with a migration hint toward the current Turbo Vision facade.

## Which API should I use?

Read [Two application models](../README.md#two-application-models) on the Std.Tui hub first. Summary:

| Goal | API surface | Minimum setup |
| --- | --- | --- |
| Widgets (dialogs, buttons, menus) | Turbo Vision `Application.Create*` handles, `AddChild`, `OnCommand`, `Run` | Create at least one widget handle, register `OnCommand`, call `Run` |
| Custom canvas (draw every cell yourself) | `Application.Configure` + `ApplicationHandlers` | Register `OnPaint` (required), optional `OnKeyPressed` / `OnResize`, draw with `Std.Console` inside `OnPaint`, call `Run` |

`Application.Run` picks the backend from session state:

- **Any** `Application.Create*` call → Turbo Vision run loop. `OnPaint` is not consulted.
- **No** Turbo Vision handles → hosted canvas loop. `OnPaint` is mandatory.

Do not call `Application.Configure` expecting it to paint over a Turbo Vision widget tree, and do not call `Application.CreateButton` in a canvas app — the two models do not compose.

## Current API

| Symbol | Description |
| --- | --- |
| `Application.Open(): Application` | Open a logical TUI application handle. Terminal ownership is acquired by `Application.Run`. |
| `Application.OpenForTest(Width, Height): Application` | Open a headless test application. |
| `Application.Close(App)` | Close an open application handle. |
| `Application.CloseForTest(App)` | Close a headless test application. |
| `Application.Size(App): Size` | Return the current application size. |
| `Application.RequestRedraw(App)` | Mark the application as needing a redraw. |
| `Application.Configure(App, Handlers)` | Store an `ApplicationHandlers` bundle. |
| `Application.Run(App)` | Run the active backend. Turbo Vision objects use the Turbo Vision path; otherwise the hosted global-handler loop runs. |
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
| `Application.AddChild(App, Parent, Child)` | Attach a button, static text, memo, text viewer, input line, list box, check box, or radio button child to a dialog or window parent. |
| `Application.SetText(App, Control, Text)` | Replace the text of a text-bearing control (button, static text, memo, text viewer, input line, check box, radio button) at runtime. Re-renders live and in headless queries. Not supported for list boxes. |
| `Application.SetChecked(App, Control, Checked)` | Set check box checked state or radio button selected state at runtime. Radio selection clears siblings in the same group. |
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
| `Application.OnKey(App, Handler)` | Optional Turbo Vision hook: `function (Application, Std.Console.KeyEvent): boolean` for keys left unhandled by the widget tree. Return `true` to mark the key consumed. |
| `Application.OnMouse(App, Handler)` | Optional Turbo Vision hook: `procedure (Application, Std.Console.Event)` for mouse events left unhandled by the widget tree. |
| `Application.Pump(App): integer` | Process one headless Turbo Vision pump step. |
| `Application.TestClickButton(App, Button)` | Queue a test click for a Turbo Vision button. |
| `Application.TestDispatchMenuCommand(App, MenuBar, MenuIndex, ItemIndex)` | Queue a menu item command for headless tests. |
| `Application.TestPump(App)` | Process one headless test step. |
| `Application.TestPumpUntilIdle(App)` | Process headless test work until idle. |
| `Application.TestSendKey(App, Key)` | Queue a key event for tests. |
| `Application.TestSendMouse(App, Event)` | Queue a mouse event for tests. |
| `Application.TestMoveMouse(App, X, Y)` | Queue a mouse move for tests. |
| `Application.TestClickMouse(App, X, Y)` | Queue a mouse click for tests. |
| `Application.TestResize(App, Width, Height)` | Queue a resize for tests. |
| `Application.TestPaste(App, Text)` | Queue pasted text for tests. |
| `Application.TestFocus(App, Gained)` | Queue a focus event for tests. |
| `Application.TestSetFileDialogResult(App, Result)` | Queue the `Option of string` returned by the next headless `Application.RunFileDialog` call. |
| `Application.TestSetDialogResult(App, Command)` | Queue the closing command returned by the next headless `Application.ExecDialog` call. |

## Screen Queries

The following query functions remain public for headless tests:

| Symbol | Description |
| --- | --- |
| `Application.QueryScreenSize(App): Size` | Return the headless screen size. |
| `Application.QueryScreenLine(App, Y): string` | Return one screen line (`Y` is one-based, `1..=height`). |
| `Application.QueryScreenCell(App, X, Y): ScreenCell` | Return one screen cell (`X`/`Y` one-based, within `QueryScreenSize`). |

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
- [Std index](../../README.md)
