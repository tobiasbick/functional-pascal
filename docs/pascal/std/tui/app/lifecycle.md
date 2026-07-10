# Std.Tui lifecycle

`Application.Open` creates a logical application handle. It does not acquire terminal state by itself.

`Application.Run(App)` starts the active backend and closes the application handle before it returns successfully. Code must not call `Application.Close(App)` again after a successful `Run`.

Use `Application.Quit(App)` from `OnCommand` to request loop exit.

Headless tests use `Application.OpenForTest`, test event helpers, and `Application.CloseForTest`.

## Interactive terminal session

On an interactive terminal, the VM keeps one upstream turbo-vision application for the lifetime of a Pascal `Application.Open` … `Application.Close` pair (or until the next `Application.Open` resets state). The first call among `Application.Run`, `Application.ExecView`, `Application.MessageBox`, or `Application.RunFileDialog` that needs the terminal creates that instance; later calls reuse it. Terminal shutdown runs once on `Application.Close`.

`Application.Run` drives the event loop on that instance. `ExecView`, `MessageBox`, and `RunFileDialog` may run while `Run` is active (nested modals).

Headless tests (`Application.OpenForTest`) do not use the live session; file-dialog results come from `Application.TestSetFileDialogResult`, and the optional `Application.TestSetDialogResult` stub applies to headless `MessageBox` when queued.

## See Also

- [Application](README.md)
- [Session API](../session.md)
- [Dialogs and windows](modals.md)
- [Handlers](handlers.md)
- [Native testing](testing.md)
- [File dialog](file-dialog.md)
