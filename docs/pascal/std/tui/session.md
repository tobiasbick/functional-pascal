# Std.Tui session API

`Std.Tui` provides an opaque `Application` handle for terminal UI sessions.

## Quick Reference

| Symbol | Description |
| --- | --- |
| `Application.Open(): Application` | Create a logical application handle. |
| `Application.OpenForTest(Width, Height): Application` | Create a headless test application. |
| `Application.Close(App)` | Close a logical application handle. |
| `Application.CloseForTest(App)` | Close a headless test application. |
| `Application.Size(App): Size` | Return the current size. |
| `Application.Run(App)` | Run the Turbo Vision event loop and close the app on success. |
| `Application.Quit(App)` | Request that the running backend exits. |

`Application.Open` returns immediately and does not acquire terminal state. Backend-specific terminal ownership starts in `Application.Run`, `Application.ExecDialog`, or `Application.RunFileDialog` — whichever needs the terminal first. Those calls reuse one upstream turbo-vision application until `Application.Close`. See [Application lifecycle](app/lifecycle.md).

After a successful `Application.Run(App)`, the runtime has already closed `App`; calling `Application.Close(App)` again is a runtime error.

## Example

```pascal
program TuiButtonTest;

uses Std.Tui, Std.Test;

mutable var
  mutable var LastCommand: integer := -1;

function Bounds(X: integer; Y: integer; Width: integer; Height: integer): Rect;
begin
  return record x := X; y := Y; width := Width; height := Height; end
end;

procedure OnCommand(App: Application; CommandId: integer);
begin
  LastCommand := CommandId;
  Application.Quit(App)
end;

begin
  var App: Application := Application.OpenForTest(40, 12);
  var DialogHandle: Dialog := Application.CreateDialog(App, Bounds(2, 1, 24, 8), 'Demo');
  var ButtonHandle: Button := Application.CreateButton(App, Bounds(4, 4, 10, 2), 'Quit', Command.Quit);
  Application.AddChild(App, DialogHandle, ButtonHandle);
  Application.OnCommand(App, OnCommand);
  Application.TestClickButton(App, ButtonHandle);
  Application.Run(App);
  AssertEquals(Command.Quit, LastCommand)
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
- [Std index](../README.md)
