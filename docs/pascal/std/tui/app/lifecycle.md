# Std.Tui lifecycle

`Application.Open` creates a logical application handle. It does not acquire terminal state by itself.

`Application.Run(App)` starts the active backend and closes the application handle before it returns successfully. Code must not call `Application.Close(App)` again after a successful `Run`.

Use `Application.Quit(App)` from callbacks to request loop exit.

Headless tests use `Application.OpenForTest`, test event helpers, and `Application.CloseForTest`.

## See Also

- [Application](README.md)
- [Session API](../session.md)
- [Native testing](testing.md)
