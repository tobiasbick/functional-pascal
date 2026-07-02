# Std.Tui native testing

Headless Turbo Vision tests use `Application.OpenForTest`, widget construction, `Application.TestClickButton`, `Application.TestClickMouse`, or `Application.TestDispatchMenuCommand`, and `Application.Run`.

Example:

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

Regression tests live under `tests/tui/controls/` (`tui_turbo_vision_*_test.fpas`, `tui_run_path_test.fpas`).

To assert painted terminal output after `Application.Pump`, add `uses Std.Console` and call [`Std.Test`](../../testing/test.md) `AssertScreenLine` or `AssertScreenCell` on the virtual CRT back buffer. Use `Application.TestClickMouse(App, X, Y)` with screen coordinates that match the painted check box or radio button marker cell.

File dialog headless example:

```pascal
Application.TestSetFileDialogResult(App, Some('selected.txt'));
var Path: option of string := Application.RunFileDialog(
  App,
  Bounds(10, 5, 50, 14),
  'Open File',
  '*',
  None
);
AssertEquals('selected.txt', Unwrap(Path))
```

Modal dialog headless example:

```pascal
Application.TestSetDialogResult(App, Command.Accept);
var CloseResult: DialogResult := Application.ExecDialog(App, DialogHandle);
AssertEquals(Command.Accept, CloseResult.command);
AssertEquals('seeded', Application.InputText(App, NameInput))
```

## See Also

- [Application](README.md)
- [Std.Test](../../testing/test.md)
