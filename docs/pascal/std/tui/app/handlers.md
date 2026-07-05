# Std.Tui handlers

Turbo Vision applications register callbacks on the `Application` handle. Commands are the primary input channel; optional hooks cover keyboard and mouse events that the widget tree does not consume.

## `Application.OnCommand`

```pascal
procedure OnCommand(App: Application; CommandId: integer);
begin
  if CommandId = Command.Quit then
    Application.Quit(App)
end;

Application.OnCommand(App, OnCommand);
```

`Application.OnCommand` expects `procedure (Application, integer)`. Use `Command.Accept`, `Command.Cancel`, `Command.Close`, and `Command.Quit` for standard actions, or application-defined positive integers for custom commands.

On an interactive terminal you may call `Application.ExecDialog` or `Application.RunFileDialog` from inside `OnCommand` while `Application.Run` is active. The runtime reuses the same upstream turbo-vision session, so menu bar and status line stay visible (see [Dialogs and windows](modals.md) and [Lifecycle](lifecycle.md)).

## Optional raw input hooks

When a keyboard or mouse event survives Turbo Vision `handle_event` without being cleared, the interactive run loop can forward it to optional hooks:

```pascal
function OnKey(App: Application; Key: Std.Console.KeyEvent): boolean;
begin
  return false
end;

procedure OnMouse(App: Application; Event: Std.Console.Event);
begin
end;

Application.OnKey(App, OnKey);
Application.OnMouse(App, OnMouse);
```

- `Application.OnKey` expects `function (Application, Std.Console.KeyEvent): boolean`. Return `true` to mark the key consumed for the rest of the loop turn.
- `Application.OnMouse` expects `procedure (Application, Std.Console.Event)`.

Custom fullscreen terminal programs that paint every cell themselves belong in [`Std.Console`](../../console/README.md), not in these hooks.

## See Also

- [Application](README.md)
- [Dialogs and windows](modals.md)
- [Types](types.md) (`Command` constants)
- [Lifecycle](lifecycle.md)
- [Session API](../session.md)
