# Std.Tui handlers

`ApplicationHandlers` is a record for bundled event handlers used by `Application.Configure`.

The explicit old `Application.HostRegisterOn*` registration helpers are no longer public. For the Turbo Vision spike, command callbacks use:

```pascal
procedure OnCommand(App: Application; CommandId: integer);
begin
  Application.Quit(App)
end;

Application.OnCommand(App, OnCommand);
```

`Application.OnCommand` expects `procedure (Application, integer)`.

`ApplicationHandlers` remains registered for the transition runtime. Its optional fields use `Some(Handler)` or `None`.

## See Also

- [Application](README.md)
- [Session API](../session.md)
