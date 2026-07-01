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

### Optional raw input hooks (Turbo Vision path)

Commands are the primary Turbo Vision input channel. When a keyboard or mouse event survives `handle_event` without being cleared, the interactive run loop can forward it to optional hooks registered on the application:

```pascal
function OnKey(App: Application; Key: Std.Console.KeyEvent): boolean;
begin
  { return true when the key is consumed }
  OnKey := false
end;

procedure OnMouse(App: Application; Event: Std.Console.Event);
begin
end;

Application.OnKey(App, OnKey);
Application.OnMouse(App, OnMouse);
```

- `Application.OnKey` expects `function (Application, Std.Console.KeyEvent): boolean`. Return `true` to mark the key consumed for the rest of the loop turn.
- `Application.OnMouse` expects `procedure (Application, Std.Console.Event)`.

These hooks are separate from `Application.Configure` (`OnKeyPressed`, `OnMouse`, …), which apply only to the hosted canvas loop.

## See Also

- [Application](README.md)
- [Types](types.md) (`Command` constants)
- [Session API](../session.md)
