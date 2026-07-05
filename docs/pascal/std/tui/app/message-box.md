# Message box

`Application.MessageBox` shows a standard Borland-style modal message box backed by upstream `turbo_vision::helpers::msgbox::message_box`. Use it for About boxes, OK prompts, and Yes/No/Cancel confirmations without building a custom dialog.

Custom layouts with read-back widgets still use [`CreateDialog`](modals.md#custom-modal-layout) and [`ExecDialog`](modals.md).

## Signature

```pascal
function Application.MessageBox(App: Application; Message: string; Options: integer): integer;
```

Returns the closing command id. OK buttons map to [`Command.Accept`](types.md#command-constants) (`10`, Borland `cmOK`). Cancel maps to [`Command.Cancel`](types.md#command-constants) (`11`). Yes/No map to upstream `cmYes` / `cmNo` (also available as `Command.Accept` / `Command.Cancel` where the helper emits those ids).

`Options` combines a **type** flag with one or more **button** flags. Combine flags with `+` when the bit patterns do not overlap.

## `MessageBoxOption` constants

Values match turbo-vision 2.0 `helpers/msgbox.rs`.

### Dialog type (pick one)

| Constant | Value | Use |
| --- | --- | --- |
| `MessageBoxOption.Warning` | `0` | Warning icon and frame |
| `MessageBoxOption.Error` | `1` | Error icon and frame |
| `MessageBoxOption.Information` | `2` | Information icon and frame |
| `MessageBoxOption.Confirmation` | `3` | Confirmation icon and frame |
| `MessageBoxOption.About` | `4` | About box (multi-line message, scroll when needed) |

### Buttons (combine with `+`)

| Constant | Value |
| --- | --- |
| `MessageBoxOption.YesButton` | `256` (`0x0100`) |
| `MessageBoxOption.NoButton` | `512` (`0x0200`) |
| `MessageBoxOption.OkButton` | `1024` (`0x0400`) |
| `MessageBoxOption.CancelButton` | `2048` (`0x0800`) |

### Presets

| Constant | Value | Buttons |
| --- | --- | --- |
| `MessageBoxOption.OkCancel` | `3072` | OK + Cancel |
| `MessageBoxOption.YesNoCancel` | `3840` | Yes + No + Cancel |

## Examples

About box (same options as the FPAS IDE):

```pascal
var CloseCommand: integer := Application.MessageBox(
  App,
  'FPAS IDE' + #10 + #10 + 'Functional Pascal IDE',
  MessageBoxOption.About + MessageBoxOption.OkButton
);
if CloseCommand = Command.Accept then
begin
  { user dismissed with OK }
end
```

Information + OK:

```pascal
Application.MessageBox(App, 'Saved.', MessageBoxOption.Information + MessageBoxOption.OkButton);
```

## Live session

On an interactive terminal, `MessageBox` runs on the same upstream turbo-vision application as [`Application.Run`](lifecycle.md). The menu bar and status line from the running session stay visible behind the modal.

You may call `MessageBox` from `OnCommand` while `Run` is active (for example Help → About in the IDE).

## Headless tests

`Application.OpenForTest` does not open a live turbo-vision session. Queue the closing command with [`Application.TestSetDialogResult`](testing.md) before `MessageBox` (same queue as `ExecDialog`).

```pascal
Application.TestSetDialogResult(App, Command.Accept);
var Cmd: integer := Application.MessageBox(App, 'Hello', MessageBoxOption.About + MessageBoxOption.OkButton);
AssertEquals(Command.Accept, Cmd);
```

## See also

- [Dialogs and windows](modals.md)
- [Handlers](handlers.md) — IDE About flow
- [Native testing](testing.md)
- [Application types](types.md)
