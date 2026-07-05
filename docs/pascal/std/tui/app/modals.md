# Std.Tui dialogs and windows

Turbo Vision dialogs and windows are opaque handles created before `Application.Run`.

| Symbol | Description |
| --- | --- |
| `Dialog` | Modal or modeless dialog handle. |
| `DialogResult` | Result of `Application.ExecDialog`; field `command` holds the closing command id. |
| `Window` | Desktop window handle. |
| `Application.CreateDialog(App, Bounds, Title): Dialog` | Create a dialog. |
| `Application.CreateWindow(App, Bounds, Title): Window` | Create a window. |
| `Application.ExecDialog(App, Dialog): DialogResult` | Run a dialog modally. Returns the command that closed it. |
| `Application.InputText(App, Field): string` | Read the text of an `InputLine` after `ExecDialog`. |
| `Application.Checked(App, Field): boolean` | Read the checked state of a `CheckBox` after `ExecDialog`. |
| `Application.Selected(App, Field): boolean` | Read the selected state of a `RadioButton` after `ExecDialog`. |
| `Application.ListSelection(App, ListBox): integer` | Read the zero-based selected list-box index after `ExecDialog`, or `-1` when no item is selected. |
| `Application.AddWindow(App, Window)` | Place a window on the application desktop. |
| `Application.AddChild(App, Parent, Child)` | Attach a control child to a dialog or window parent. |

`Parent` may be a `Dialog` or `Window`. `Child` may be a `Button`, `StaticText`, `Memo`, `TextViewer`, `InputLine`, `ListBox`, `CheckBox`, or `RadioButton`.

Use `Application.ExecDialog` for modal dialogs with read-back (`InputText`, `Checked`, `Selected`, `ListSelection`).

## Custom modal layout

Build the dialog with `CreateDialog`, `AddChild`, and related APIs, then run it with `Application.ExecDialog`. The FPAS IDE About box (`apps/ide/src/dialog/about.fpas`) is implemented this way today: manual bounds, `TextViewer`, and an OK button bound to `Command.Accept`.

There is no public `Application.MessageBox` symbol yet. Standard Borland message boxes (About, OK, Yes/No) are implemented upstream in `turbo_vision::helpers::msgbox`; bridge work to call them from Rust is tracked in [docs/refactor/tui-bridge/02-about-message-box.md](../../../refactor/tui-bridge/02-about-message-box.md). A future optional Pascal wrapper is described in [07-pascal-message-box-api.md](../../../refactor/tui-bridge/07-pascal-message-box-api.md).

## Interactive session

On an interactive terminal, `Application.ExecDialog` runs on the same upstream turbo-vision application instance as `Application.Run`. The menu bar and status line from the running session stay visible while the modal is open.

You may call `Application.ExecDialog` from `OnCommand` while `Run` is active (for example Help → About). The run loop dispatches Pascal handlers without holding the upstream application, so the modal can execute on the shared session.

Headless `Application.OpenForTest` sessions do not open a live turbo-vision application. Queue the closing command with `Application.TestSetDialogResult` before `Application.ExecDialog`.

### IDE About tests

`apps/ide/tests/shell/about_menu_test.fpas` dispatches Help → About with `TestDispatchMenuCommand` and queues `Command.Accept` before `Run`. `apps/ide/tests/dialog/dialog_test.fpas` calls `HandleCommand` directly with `CmdHelpAbout` (`100`). After upstream `message_box` lands ([refactor 02](../../../refactor/tui-bridge/02-about-message-box.md)), headless tests may still use `TestSetDialogResult` until the headless path is unified ([03](../../../refactor/tui-bridge/03-headless-test-util.md)).

## See Also

- [Application](README.md)
- [Application lifecycle](lifecycle.md)
- [Handlers](handlers.md) — `ExecDialog` from `OnCommand` during `Run`
- [Controls](controls.md)
- [Types](types.md) — `Command.Accept`, menu `CM_ABOUT` (`100`)
