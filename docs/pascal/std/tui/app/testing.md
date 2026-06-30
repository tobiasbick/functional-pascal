# Std.Tui native testing

Headless TUI tests use `Application.OpenForTest` and the `Application.Test*` helpers.

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
  var Dialog: TuiDialog := Application.CreateDialog(App, Bounds(2, 1, 24, 8), 'Demo');
  var Button: TuiButton := Application.CreateButton(App, Bounds(4, 4, 10, 2), 'Quit', 77);
  Application.AddChild(App, Dialog, Button);
  Application.OnCommand(App, OnCommand);
  Application.TestClickButton(App, Button);
  Application.Run(App);
  AssertEquals(77, LastCommand)
end.
```

Regression tests live under `tests/tui/`.

## See Also

- [Application](README.md)
- [Std.Test](../../testing/test.md)
