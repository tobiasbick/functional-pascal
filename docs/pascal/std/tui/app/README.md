# Std.Tui application

`Std.Tui.Application` is the Turbo Vision application facade for terminal UI code, implemented over [`turbo-vision`](https://crates.io/crates/turbo-vision) 2.0 from [turbo-vision-4-rust](https://github.com/aovestdipaperino/turbo-vision-4-rust).

## Minimum setup

1. `Application.Open` or `Application.OpenForTest`
2. Create views with `Dialog.NewModal`, `Button.New`, `Window.New`, … (see [Controls](controls.md) and [Dialogs and windows](modals.md))
3. Attach children with `Dialog.Add` / `Window.Add`; place windows with `Desktop.Add`
4. Optional: `Application.OnKey` / `Application.OnMouse` before `Run`, or bundle handlers with `Application.Configure`
5. `Application.Run(App, OnCommand)` or `Application.Configure` then `Application.Run(App)` — blocking event loop

On an interactive terminal, `Run`, `ExecView`, `MessageBox`, and `RunFileDialog` share one upstream turbo-vision application for the `Open` … `Close` lifetime. See [Lifecycle](lifecycle.md).

Custom modal layout (`Dialog.NewModal` + `ExecView`) is for dialogs with read-back or app-specific widgets. Standard Borland message boxes use [`Application.MessageBox`](message-box.md).

## `Application.*` symbols

| Symbol | Description |
| --- | --- |
| `Application.New(): Application` | Alias for `Application.Open`. |
| `Application.Open(): Application` | Open a logical TUI application handle. |
| `Application.OpenForTest(Width, Height): Application` | Open a headless test application. |
| `Application.Close(App)` | Close an open application handle. |
| `Application.CloseForTest(App)` | Close a headless test application. |
| `Application.Size(App): Size` | Return the current application size. |
| `Application.Run(App)` | Run the event loop using a handler from `Application.Configure` (see [Handlers](handlers.md)). |
| `Application.Run(App, OnCommand)` | Run with `procedure (Application, integer)`. |
| `Application.Configure(App, Handlers)` | Install bundled `ApplicationHandlers` (Graph-style hosted dispatch). |
| `Application.Quit(App)` | Request that `Application.Run` exits. |
| `Application.ExecView(App, Dialog): integer` | Run a modal dialog or window view; returns the closing command id. |
| `Application.MessageBox(App, Message, Options): integer` | Show an upstream message box. See [Message box](message-box.md). |
| `Application.RunFileDialog(App, Bounds, Title, Wildcard, StartPath): Option of string` | Modal file picker. See [File dialog](file-dialog.md). |
| `Desktop.Add(App, Window)` | Place a modeless window on the desktop. |
| `Application.SetMenuBar(App, MenuBar)` | Attach a menu bar to the live session. |
| `Application.SetStatusLine(App, StatusLine)` | Attach a status line to the live session. |
| `Application.OnKey(App, Handler)` | Optional: `function (Application, Std.Console.KeyEvent): boolean`. |
| `Application.OnMouse(App, Handler)` | Optional: `procedure (Application, Std.Console.Event)`. |
| `Application.TestClickMouse(App, X, Y)` | Headless: left-click at screen coordinates. |
| `Test.Click(App, Button)` | Headless: queue a button click. |
| `Test.DispatchMenu(App, MenuBar, MenuIndex, ItemIndex)` | Headless: dispatch a menu item command id. |
| `Test.InjectCommand(App, Command)` | Headless: inject a command during `Run` tests. |
| `Test.InjectKeyboard(App, KeyCode)` | Headless: inject a keyboard event. |
| `Application.TestSetFileDialogResult(App, Result)` | Headless: queue the next `RunFileDialog` result. |
| `Application.TestSetDialogResult(App, Command)` | Headless: queue the closing command for the next `MessageBox` (stub path). |

View factories (`Button.New`, `Dialog.Add`, `InputLine.Text`, …) are documented in [Controls](controls.md) and [Dialogs and windows](modals.md).

For headless screen assertions, add `uses Std.Console` and call [`Std.Test`](../../testing/test.md) `AssertScreenLine` / `AssertScreenCell` on the virtual CRT back buffer.

## See Also

- [Session API](../session.md)
- [Application types](types.md)
- [Controls](controls.md)
- [Dialogs and windows](modals.md)
- [Message box](message-box.md)
- [Handlers](handlers.md)
- [Lifecycle](lifecycle.md)
- [Native testing](testing.md)
- [VM bridge](vm-bridge.md)
