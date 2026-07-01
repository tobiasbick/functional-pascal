# Std.Tui handlers

`ApplicationHandlers` is a record for bundled event handlers used by `Application.Configure`. Optional fields use `Some(Handler)` or `None`.

These handlers apply only to the **hosted canvas** run loop. See [Two application models](../README.md#two-application-models): if the session contains any Turbo Vision widget handle, `Application.Run` uses the Turbo Vision path and **does not** call `OnPaint`, `OnKeyPressed`, or the other `Application.Configure` handlers.

Turbo Vision apps register command callbacks with `Application.OnCommand`:

```pascal
procedure OnCommand(App: Application; CommandId: integer);
begin
  if CommandId = Command.Quit then
    Application.Quit(App)
end;

Application.OnCommand(App, OnCommand);
```

`Application.OnCommand` expects `procedure (Application, integer)`. Use `Command.Accept`, `Command.Cancel`, `Command.Close`, and `Command.Quit` for standard actions, or application-defined positive integers for custom commands.

## See Also

- [Application](README.md)
- [Types](types.md) (`Command` constants)
- [Session API](../session.md)
