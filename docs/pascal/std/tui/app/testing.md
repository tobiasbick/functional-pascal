# Std.Tui native testing

Headless Turbo Vision tests use `Application.OpenForTest`, Turbo Vision view construction, `Std.Tui.Test.*` helpers, and `Application.Run`.

Example:

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
  var Cmd: integer := Application.ExecView(App, Dlg);
  AssertEquals(CM_QUIT, Cmd);
  Application.CloseForTest(App)
end.
```

Regression tests live under `tests/tui/views/`, `tests/tui/smoke/`, `tests/tui/modals/`, and `tests/tui/events/`. The retained `apps/ide/` source is legacy and has no test suite.

## Headless test helpers (`Std.Tui.Test.*`)

Use `uses Std.Tui` (and `Std.Test` when asserting). Short names such as `Test.Click` resolve inside the TUI unit.

| Symbol | Description |
| --- | --- |
| `Test.Click(App, Button)` | Queue a headless button click at the button center. |
| `Test.DispatchMenu(App, MenuBar, MenuIndex, ItemIndex)` | Dispatch a menu item command id from menu bar data. |
| `Test.InjectCommand(App, Command)` | Queue a command for the next headless `Application.Run` turn. |
| `Test.InjectKeyboard(App, KeyCode)` | Queue a Turbo Vision key code for the next run-loop turn. |

Additional helpers on `Application` (stub queues and coordinate clicks):

| Symbol | Description |
| --- | --- |
| `Application.TestClickMouse(App, X, Y)` | Left-click at screen coordinates (check box / radio marker cells). |
| `Application.TestSetDialogResult(App, Command)` | Stub queue for the next headless `MessageBox` when not driving a full modal loop. |
| `Application.TestSetFileDialogResult(App, Result)` | Stub queue for the next headless `RunFileDialog`. |

Prefer real headless modal paths (`ExecView` + `Test.Click`) over stub queues where possible.

## Headless run loop

For desktop programs that call `Application.Run` in headless mode, inject commands or keys before `Run`:

```pascal
var App: Application := Application.OpenForTest(40, 14);
Test.InjectCommand(App, CM_QUIT);
Application.Run(App, OnCommand);
```

`Test.InjectKeyboard` queues a Turbo Vision key code for the next run-loop turn (used by IDE and event tests).

When a queued keyboard or mouse event closes a modal started by a command handler, later queued application commands remain available to the outer `Application.Run` loop. This lets a test dismiss a modal and then inject `CM_QUIT` in the same sequence.

## File dialog stub

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

## Message box stub

When exercising the stub queue path (without driving a full headless modal loop):

```pascal
Application.TestSetDialogResult(App, CM_OK);
var Cmd: integer := Application.MessageBox(App, 'Hello', MessageBoxOption.About + MessageBoxOption.OkButton);
AssertEquals(CM_OK, Cmd);
```

Prefer real headless modal paths (`ExecView` + `Test.Click`) where possible.

## Screen assertions

To assert painted terminal output, add `uses Std.Console` and call [`Std.Test`](../../testing/test.md) `AssertScreenLine` or `AssertScreenCell` on the virtual CRT back buffer. Use `Application.TestClickMouse(App, X, Y)` with screen coordinates that match the painted check box or radio button marker cell.

## See Also

- [Application](README.md)
- [Dialogs and windows](modals.md)
- [Handlers](handlers.md)
- [Std.Test](../../testing/test.md)
