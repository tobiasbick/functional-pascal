# Std.Tui handlers

`ApplicationHandlers` is a record for bundled event handlers used by `Application.Configure`. Optional fields use `Some(Handler)` or `None`.

The hosted global-handler loop still supports `OnPaint`, `OnKeyPressed`, `OnResize`, and related handlers for apps that do not construct Turbo Vision widgets.

Turbo Vision apps should register command callbacks with `Application.OnCommand`:

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
