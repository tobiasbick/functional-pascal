# Std.Tui session API

`Std.Tui` provides an opaque `Application` handle for terminal UI sessions.

## Quick Reference

| Symbol | Description |
| --- | --- |
| `Application.Open(): Application` | Create a logical application handle (`Application.New` is an alias). |
| `Application.OpenForTest(Width, Height): Application` | Create a headless test application. |
| `Application.Close(App)` | Close a logical application handle. |
| `Application.CloseForTest(App)` | Close a headless test application. |
| `Application.Size(App): Size` | Return the current size. |
| `Application.Run(App)` | Run the Turbo Vision event loop (requires `Application.Configure` or a registered handler). |
| `Application.Run(App, OnCommand)` | Run with an inline `procedure (Application, integer)` handler. |
| `Application.Configure(App, Handlers)` | Install bundled `ApplicationHandlers` before `Run(App)`. |
| `Application.Quit(App)` | Request that the running loop exits. |

`Application.Open` returns immediately and does not acquire terminal state. Backend-specific terminal ownership starts in `Application.Run`, `Application.ExecView`, `Application.MessageBox`, or `Application.RunFileDialog` — whichever needs the terminal first. Those calls reuse one upstream turbo-vision application until `Application.Close`. See [Application lifecycle](app/lifecycle.md).

After a successful `Application.Run(App)`, the runtime has already closed `App`; calling `Application.Close(App)` again is a runtime error.

## Example

```pascal
program TuiButtonTest;

uses Std.Tui, Std.Test;

procedure OnCommand(App: Application; Cmd: integer);
begin
  AssertEquals(CM_QUIT, Cmd);
  Application.Quit(App)
end;

function Bounds(X: integer; Y: integer; Width: integer; Height: integer): TuiRect;
begin
  return record x := X; y := Y; width := Width; height := Height; end
end;

begin
  var App: Application := Application.OpenForTest(40, 12);
  var Dlg: Dialog := Dialog.NewModal(Bounds(2, 1, 24, 8), 'Demo');
  var Btn: Button := Button.New(Bounds(4, 4, 10, 2), 'Quit', CM_QUIT, false);
  Dialog.Add(Dlg, Btn);
  Test.Click(App, Btn);
  Application.Run(App, OnCommand);
  Application.CloseForTest(App)
end.
```

## Implementation

| Concern | Location |
| --- | --- |
| Sema registry | [`loaded/tui/mod.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/tui/mod.rs) |
| Compiler lowering | [`std_calls/tui/`](../../../../crates/fpas-compiler/src/compiler/std_calls/tui/) |
| VM execution | [`execute/io/tui/`](../../../../crates/fpas-vm/src/vm/execute/io/tui/) |
| Runtime session | [`tui/session/`](../../../../crates/fpas-std/src/tui/session/) |

## See Also

- [Terminal UI index](README.md)
- [Application](app/README.md)
- [Dialogs and windows](app/modals.md)
- [Application lifecycle](app/lifecycle.md)
- [Std index](../README.md)
