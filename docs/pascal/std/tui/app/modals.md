# Std.Tui dialogs and windows

Turbo Vision dialogs and windows are opaque handles created before `Application.Run`.

| Symbol | Description |
| --- | --- |
| `Dialog.NewModal(Bounds, Title): Dialog` | Create a modal dialog for `Application.ExecView`. |
| `Dialog.Add(Dlg, Child)` | Attach a control child to a dialog. |
| `Dialog.SetTitle(Dlg, Title)` | Replace dialog title text. |
| `Window.New(Bounds, Title): Window` | Create a desktop window. |
| `Window.Add(Win, Child)` | Attach a control child to a window. |
| `Window.SetTitle(Win, Title)` | Replace window title text. |
| `Desktop.Add(App, Window)` | Place a window on the application desktop. |
| `Application.ExecView(App, Dialog): integer` | Run a modal view; returns the closing command id. |

`Child` may be a `Button`, `StaticText`, `Memo`, `TextViewer`, `InputLine`, `ListBox`, `Outline`, `CheckBox`, or `RadioButton`.

After `ExecView`, read control state with `InputLine.Text`, `CheckBox.Checked`, `RadioButton.Selected`, `ListBox.Selection`, `Outline.Selection`, and `Outline.SelectedText`.

## Custom modal layout

Build the dialog with `Dialog.NewModal`, `Dialog.Add`, and related APIs, then run it with `Application.ExecView`.

For standard Borland message boxes (About, OK, Yes/No), use [`Application.MessageBox`](message-box.md). The FPAS IDE About flow calls it with `MessageBoxOption.About + MessageBoxOption.OkButton`.

## Interactive session

On an interactive terminal, `Application.ExecView` and `Application.MessageBox` run on the same upstream turbo-vision application instance as `Application.Run`. The menu bar and status line from the running session stay visible while the modal is open.

You may call `ExecView` or `MessageBox` from `OnCommand` while `Run` is active (for example Help → About).

## Headless tests

Headless `Application.OpenForTest` sessions paint modals through upstream `draw` without a live terminal loop. Use `Test.Click` before `ExecView`, or `Application.TestSetDialogResult` before `MessageBox` when exercising the stub queue path. See [Native testing](testing.md).

## See Also

- [Application](README.md)
- [Application lifecycle](lifecycle.md)
- [Handlers](handlers.md)
- [Message box](message-box.md)
- [Controls](controls.md)
- [Types](types.md)
