# Std.Tui native testing

Headless Turbo Vision tests use `Application.OpenForTest`, try-2 view construction, test injection helpers, and `Application.Run`.

Example:

```pascal
program TuiButtonTest;

uses Std.Tui, Std.Test;

procedure OnCommand(App: Application; Cmd: integer);
begin
  AssertEquals(CM_QUIT, Cmd);
  Application.Quit(App)
end;

function Bounds(X: integer; Y: integer; Width: integer; Height: integer): Rect;
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

Regression tests live under `tests/tui/views/`, `tests/tui/smoke/`, `tests/tui/modals/`, `tests/tui/events/`, and `apps/ide/tests/`.

## Interim vs target test API

The registered surface today uses `Application.Test*` names. Phase 7 closure will introduce `Std.Tui.Test.*` helpers and retire the interim names (see [remaining-work.md](../../../refactor-tui-try-2/remaining-work.md) stream B).

| Interim (current) | Target (planned) | Status |
| --- | --- | --- |
| `Application.TestClickButton` | `Std.Tui.Test.Click` | **Landed** — both names compile to the same intrinsic |
| `Application.TestClickMouse` | covered by `Test.InjectEvent` (future) or retained coordinate helper |
| `Application.TestInjectKeyboard` | `Std.Tui.Test.InjectKeyboard` | **Landed** — both names compile to the same intrinsic |
| `Application.TestInjectCommand` | `Std.Tui.Test.InjectCommand` | **Landed** — both names compile to the same intrinsic |
| `Application.TestDispatchMenuCommand` | `Std.Tui.Test.DispatchMenu` | **Landed** — both names compile to the same intrinsic |
| `Application.TestSetDialogResult` | prefer real headless modal loops |
| `Application.TestSetFileDialogResult` | prefer upstream headless execute when available |

Until migration completes, examples in this page use the preferred `Test.*` names where available; interim `Application.Test*` symbols remain registered as aliases.

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
