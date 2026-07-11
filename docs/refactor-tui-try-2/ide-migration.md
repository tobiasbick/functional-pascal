# IDE migration

Notes for rewriting [`apps/ide`](../../apps/ide) on the try-2 API. **Complete** — menu, status, shell run loop, About, Open, automated IDE tests, and manual terminal sign-off (2026-07-09) all use the try-2 API.

## Current IDE TUI usage

| Area | Files | try-2 status |
| --- | --- | --- |
| Shell / main loop | `src/shell.fpas`, `src/main.fpas` | `Application.New`, `Application.Run(App, OnCommand)`, `Application.Close` |
| Menu | `src/menu.fpas` | `MenuBar.New`, `SetMenuBar`, direct `CM_QUIT`, `CM_OPEN`, `CM_ABOUT` |
| Status | `src/status.fpas` | `StatusLine.New`, `SetStatusLine` |
| Dialogs | `src/dialog.fpas`, `src/dialog/about.fpas`, `src/dialog/open.fpas` | `Application.MessageBox`, `Application.RunFileDialog`; Open headless tests seed the interim Try-2 file dialog adapter through `TestSetFileDialogResult` |
| Theme | `src/theme.fpas` | May be layout constants only |

Tests: `apps/ide/tests/` — menu, shell, dialog, status, theme. Shell/dialog tests use `Test.InjectCommand` and `Test.InjectKeyboard`; Open uses `TestSetFileDialogResult` to seed Try-2 session state.

Latest automated sign-off, recorded 2026-07-09:

- `cargo run -q -p fpas-cli -- test apps/ide/tests/` — 7 passed.
- `cargo test -p fpas-sema std_units::tui`, `cargo test -p fpas-compiler std_library::tui`, and `cargo test -p fpas-vm tui_spec_links` all passed.
- Try-1 `tests/tui/controls/` suite removed in phase 7; IDE and try-2 regressions cover the same flows under `tests/tui/` and `apps/ide/tests/`.

## Target structure (unchanged units)

Keep thematic units; only update TUI calls:

```text
apps/ide/src/
  main.fpas
  shell.fpas          — Application.Run(App, OnCommand)
  menu.fpas           — MenuBar.New, Menu records with CM_*
  status.fpas         — StatusLine.New
  dialog/
    about.fpas        — Application.MessageBox (already thin)
    open.fpas         — RunFileDialog + MessageBox
  theme.fpas
```

## Command migration

| try-1 | try-2 |
| --- | --- |
| `Command.Quit` | `CM_QUIT` |
| `Command.Accept` | `CM_OK` |
| `Command.Cancel` | `CM_CANCEL` |
| `Command.Close` | `CM_CLOSE` |
| Menu About `100` | `CM_ABOUT` (no offset semantics) |

## Shell rewrite sketch

```pascal
unit Ide.Shell;

uses Std.Tui, Ide.Menu, Ide.Status;

procedure OnCommand(App: Application; Cmd: CommandId);

procedure Run;
begin
  var App := Application.New();
  Application.SetMenuBar(App, Ide.Menu.Build(App));
  Application.SetStatusLine(App, Ide.Status.Build(App));
  { editor window / chrome added here }
  Application.Run(App, OnCommand);
  Application.Close(App)
end;
```

## Menu rewrite sketch

```pascal
function Build(App: Application): MenuBar;
begin
  return MenuBar.New(Bounds(0, 0, 80, 1), [
    record title := '~F~ile'; items := [
      record text := '~O~pen'; commandId := CM_OPEN; end,
      record text := 'E~x~it'; commandId := CM_QUIT; end
    ]; end,
    record title := '~H~elp'; items := [
      record text := '~A~bout'; commandId := CM_ABOUT; end
    ]; end
  ])
end;
```

Handle `CM_OPEN` in `OnCommand` → `Ide.Dialog.Open.Show`.

## About dialog

Keep upstream message box (already used in try-1):

```pascal
var _: CommandId := Application.MessageBox(App, Text, MessageBoxOption.About + MessageBoxOption.OkButton);
```

No custom `CreateDialog` for About.

## Open file flow

[`open.fpas`](../../apps/ide/src/dialog/open.fpas) — minimal change:

```pascal
var Selected := Application.RunFileDialog(App, Bounds(...), 'Open File', '*', None);
```

## Custom dialogs (if any remain)

If IDE needs a non-standard dialog:

```pascal
var Dlg := Dialog.NewModal(Bounds(...), 'Title');
Dlg.Add(StaticText.New(...));
Dlg.Add(InputLine.New(...));
var Cmd := Application.ExecView(App, Dlg);
if Cmd = CM_OK then
  var Text := InputLine.Text(Field)
```

## Tests migration

| Test file | Focus |
| --- | --- |
| `tests/menu/menu_test.fpas` | Menu command dispatch with `Test.DispatchMenu` |
| `tests/shell/chrome_test.fpas` | Menu + status visible lines |
| `tests/shell/about_menu_test.fpas` | `CM_ABOUT` → message box |
| `tests/shell/open_menu_test.fpas` | `CM_OPEN` flow |
| `tests/dialog/dialog_test.fpas` | Message box / file dialog |
| `tests/status/status_test.fpas` | Status line text |

Replace:

- [x] `Application.CreateMenuBar` → `MenuBar.New`
- [x] `Test.DispatchMenu` → run-loop command injection (replaces interim menu dispatch helper)
- [x] `Command.*` → `CM_*`
- [ ] `Application.TestSetFileDialogResult` → final headless file dialog path or `Std.Tui.Test.*` during Phase 7 closure ([remaining-work.md](remaining-work.md) stream B)

## IDE-specific commands

Define IDE-only commands above `CM_USER` (4096) if needed:

```pascal
const
  CM_IDE_OPEN_PROJECT = CM_USER + 1;
  CM_IDE_BUILD = CM_USER + 2;
```

Document in `apps/ide/README` if one exists, or in source comments only.

## Manual acceptance

After automated tests pass:

1. Launch IDE in a real terminal: `cargo run -p fpas-cli -- apps/ide/ide.fpasprj`.
2. Verify File → Exit, Help → About, File → Open cancel, and File → Open confirm path.
3. Resize terminal during `Run` and confirm no panic or corrupted chrome.

Status: **signed off 2026-07-09** — File / Exit, Help / About, File / Open, and resize verified in a live terminal after the try-2 chrome menu command-id fix.

## Out of scope for IDE v1 on try-2

- Multi-window editor (`EditorWindow`) — defer to phase 8
- Syntax-highlighted editor pane
- Docking / split views

The IDE can ship with menu + status + message boxes + file dialog on try-2 without editor widgets.
