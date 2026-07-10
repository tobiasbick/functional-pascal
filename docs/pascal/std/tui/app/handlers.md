# Std.Tui handlers

Turbo Vision applications handle user input through command ids and optional keyboard/mouse hooks.

## `Application.Run(App, OnCommand)`

Primary command channel:

```pascal
procedure OnCommand(App: Application; Cmd: integer);
begin
  if Cmd = CM_QUIT then
    Application.Quit(App)
end;

Application.Run(App, OnCommand);
```

`OnCommand` must be `procedure (Application, integer)`. Use `CM_*` constants for standard actions, or application-defined positive integers for custom commands.

`Application.Run(App)` with one argument is also supported when a handler was registered through the VM host path; prefer the two-argument form in application code.

On an interactive terminal you may call `Application.ExecView`, `Application.MessageBox`, or `Application.RunFileDialog` from inside `OnCommand` while `Run` is active. The runtime reuses the same upstream turbo-vision session, so menu bar and status line stay visible (see [Dialogs and windows](modals.md) and [Lifecycle](lifecycle.md)).

### IDE About flow

Help → About in `apps/ide` dispatches menu command `CM_ABOUT` to `OnCommand`, which calls `Ide.Dialog.ShowAbout`. That procedure calls `Application.MessageBox` with `MessageBoxOption.About + MessageBoxOption.OkButton`; dismissal returns `CM_OK`. See [Message box](message-box.md).

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
Application.Run(App, OnCommand);
```

- `Application.OnKey` expects `function (Application, Std.Console.KeyEvent): boolean`. Return `true` to mark the key consumed for the rest of the loop turn.
- `Application.OnMouse` expects `procedure (Application, Std.Console.Event)`.

Custom fullscreen terminal programs that paint every cell themselves belong in [`Std.Console`](../../console/README.md), not in these hooks.

## See Also

- [Application](README.md)
- [Dialogs and windows](modals.md)
- [Types](types.md) (`CM_*` constants)
- [Lifecycle](lifecycle.md)
- [Native testing](testing.md)
- [Session API](../session.md)
