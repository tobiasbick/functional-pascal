# Std.Tui application

`Std.Tui.Application` is the application facade for terminal UI code.

The current public surface is intentionally small while the implementation moves to the Rust `turbo-vision` crate. The old retained `Application.Host*` API is not registered.
Calls to removed retained APIs such as `Application.Host*`, retained view queries, modal queries, and `Application.ShowFramedDialog` report a Sema error with a migration hint toward the current Turbo Vision facade.

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
| `Application.AddChild(App, Parent, Button)` | Attach a button to a dialog or window parent. |
| `Application.AddWindow(App, Window)` | Place a window on the application desktop. |
| `Application.OnCommand(App, Handler)` | Register `procedure (Application, integer)` for command dispatch. Use `Command.*` constants for standard actions. |
| `Application.Pump(App): integer` | Process one headless Turbo Vision pump step. |
| `Application.TestClickButton(App, Button)` | Queue a test click for a Turbo Vision button. |
| `Application.TestPump(App)` | Process one headless test step. |
| `Application.TestPumpUntilIdle(App)` | Process headless test work until idle. |
| `Application.TestSendKey(App, Key)` | Queue a key event for tests. |
| `Application.TestSendMouse(App, Event)` | Queue a mouse event for tests. |
| `Application.TestMoveMouse(App, X, Y)` | Queue a mouse move for tests. |
| `Application.TestClickMouse(App, X, Y)` | Queue a mouse click for tests. |
| `Application.TestResize(App, Width, Height)` | Queue a resize for tests. |
| `Application.TestPaste(App, Text)` | Queue pasted text for tests. |
| `Application.TestFocus(App, Gained)` | Queue a focus event for tests. |

## Screen Queries

The following query functions remain public for headless tests:

| Symbol | Description |
| --- | --- |
| `Application.QueryScreenSize(App): Size` | Return the headless screen size. |
| `Application.QueryScreenLine(App, Y): string` | Return one screen line. |
| `Application.QueryScreenCell(App, X, Y): ScreenCell` | Return one screen cell. |

## See Also

- [Session API](../session.md)
- [Application types](types.md)
- [Native testing](testing.md)
- [Std index](../../README.md)
