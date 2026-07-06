# Deletion checklist

Complete list of try-1 artifacts to remove during [phase 7](migration-phases.md#phase-7--delete-try-1-12-days). Use `rg` to confirm zero matches before closing the rewrite.

## Pascal public API (sema / docs)

### Removed symbols

- `Application.CreateDialog`, `CreateWindow`, `CreateButton`, `CreateStaticText`
- `Application.CreateMemo`, `CreateTextViewer`, `CreateInputLine`
- `Application.CreateListBox`, `CreateOutline`, `CreateCheckBox`, `CreateRadioButton`
- `Application.CreateMenuBar`, `CreateStatusLine`
- `Application.AddChild`, `AddWindow`
- `Application.ExecDialog` (replaced by `ExecView`)
- `Application.InputText`, `Checked`, `Selected`
- `Application.ListSelection`, `OutlineSelection`, `OutlineSelectedText`
- `Application.SetText`, `SetChecked`, `SetItems`, `SetOutlineNodes`, `SetTitle`
- `Application.SetMenuBar` — **keep** (same name)
- `Application.SetMenus`, `SetStatusItems` → move to `MenuBar.SetMenus`, `StatusLine.SetItems`
- `Application.Pump`
- `Application.TestClickButton`, `TestClickMouse`, `TestDispatchMenuCommand`
- `Application.TestSetFileDialogResult`, `TestSetDialogResult`
- `Command.Quit`, `Command.Close`, `Command.Accept`, `Command.Cancel`

### Replaced symbols (not deleted, renamed)

| try-1 | try-2 |
| --- | --- |
| `Application.Open` | `Application.New` or keep `Open` as alias |
| `Application.ExecDialog` | `Application.ExecView` |
| `Application.InputText` | `InputLine.Text` |
| `Application.Checked` | `CheckBox.Checked` |
| `Application.Selected` | `RadioButton.Selected` |
| `Application.ListSelection` | `ListBox.Selection` |
| `Application.AddWindow` | `Desktop.Add` |

## Bytecode intrinsics

Remove `TuiCreate*` and related try-1 variants from `fpas-bytecode` (grep `TuiCreate`, `TuiAddChild`, `TuiInputText`, `TuiPump`, `TuiTestSet`).

Add try-2 intrinsics per [upstream-mapping.md](upstream-mapping.md).

## VM modules (delete files)

```
crates/fpas-vm/src/vm/execute/io/tui/reconcile.rs
crates/fpas-vm/src/vm/execute/io/tui/live_patch.rs
crates/fpas-vm/src/vm/execute/io/tui/command_map.rs
crates/fpas-vm/src/vm/execute/io/tui/control_create.rs
crates/fpas-vm/src/vm/execute/io/tui/tv_views.rs
crates/fpas-vm/src/vm/execute/io/tui/controls.rs
crates/fpas-vm/src/vm/execute/io/tui/dialogs.rs
crates/fpas-vm/src/vm/execute/io/tui/windows.rs
crates/fpas-vm/src/vm/execute/io/tui/navigation.rs
crates/fpas-vm/src/vm/execute/io/tui/chrome_layout.rs
crates/fpas-vm/src/vm/execute/io/tui/handle_records.rs
crates/fpas-vm/src/vm/execute/io/tui/handles.rs
crates/fpas-vm/src/vm/execute/io/tui/records.rs
crates/fpas-vm/src/vm/execute/io/tui/interactive_loop.rs
crates/fpas-vm/src/vm/execute/io/tui/commands.rs
crates/fpas-vm/src/vm/execute/io/tui/test_mouse.rs
crates/fpas-vm/src/vm/execute/io/tui/outline_nodes.rs
crates/fpas-vm/src/vm/execute/io/tui/outline_read.rs
crates/fpas-vm/src/vm/execute/io/tui/callbacks.rs
crates/fpas-vm/src/vm/execute/io/tui/tv_input_events.rs
crates/fpas-vm/src/vm/execute/io/tui/exec_dialog.rs
crates/fpas-vm/src/vm/execute/io/tui/file_dialog.rs
crates/fpas-vm/src/vm/execute/io/tui/msgbox.rs
crates/fpas-vm/src/vm/execute/io/tui/lifecycle.rs
crates/fpas-vm/src/vm/execute/io/tui/headless_tv_draw.rs
crates/fpas-vm/src/vm/execute/io/tui/application.rs
crates/fpas-vm/src/vm/execute/io/tui/tui_run.rs
crates/fpas-vm/src/vm/execute/io/tui/bridged_button.rs
crates/fpas-vm/src/vm/execute/io/tui/bridged_check_box.rs
crates/fpas-vm/src/vm/execute/io/tui/bridged_list_box.rs
crates/fpas-vm/src/vm/execute/io/tui/bridged_outline.rs
crates/fpas-vm/src/vm/execute/io/tui/bridged_radio_button.rs
crates/fpas-vm/src/vm/execute/io/tui/bridged_static_text.rs
crates/fpas-vm/src/vm/execute/io/tui/bridged_memo.rs
crates/fpas-vm/src/vm/execute/io/tui/bridged_text_viewer.rs
```

Merged into new modules — delete old names after merge:

- `session_app.rs` → `session.rs`
- `tv_run.rs` → `run.rs`
- `tv_geometry.rs` → `geometry.rs`
- `menu_build.rs` → `chrome.rs`

## VM types (delete from `shared/tui.rs`)

- `TurboVisionState`
- `TurboVisionObject` enum and all `TurboVision*` snapshot structs
- `pending_reconcile`, `pending_headless_repaint`
- `live_view_ids`, `live_child_root_view_ids`
- `test_file_dialog_result`, `test_dialog_result` (if headless uses real modals)

## fpas-std

- `command_ids.rs` — `COMMAND_QUIT`, `COMMAND_OK`, etc. if superseded by `CM_*`
- Any `TurboVisionBoolCell` usage only for reconcile — remove if unused

Grep patterns that must return **no hits** in `crates/`:

```text
TurboVisionObject
pending_reconcile
turbo_vision_reconcile
FPAS_TV_COMMAND_OFFSET
bridged_
TuiCreateDialog
TuiAddChild
COMMAND_QUIT
reserved_list_matches_upstream
```

## Tests (delete directory when empty)

```text
tests/tui/controls/    — entire directory after porting to tests/tui/views/
```

37 try-1 files — delete each when try-2 replacement lands; remove directory in phase 7.

## Examples

Rewrite to try-2 API:

```text
examples/pascal/tui/turbo_vision_dialog.fpas
examples/pascal/tui/turbo_vision_window.fpas
examples/pascal/tui/exec_dialog.fpas
examples/pascal/tui/runtime_setters.fpas
examples/pascal/tui/turbo_vision_outline.fpas
examples/pascal/tui/message_box.fpas
```

## Documentation to replace

Remove or fully rewrite under `docs/pascal/std/tui/`:

- `app/README.md` — `Application.Create*` table
- `app/types.md` — `Command.*`, offset band section
- `app/vm-bridge.md` — 40-module table, reconcile architecture
- `app/controls.md`, `modals.md`, `handlers.md` — align with [target-api.md](target-api.md)

Keep until rewritten:

- `cell-width.md` — if still accurate for CRT
- `terminal-checklist.md` — update commands only

## Planning docs

When done:

- Delete `docs/refactor-tui-try-2/` **or** add banner at top of README: `Status: completed — see docs/pascal/std/tui/`
- Remove entry from `docs/future/README.md` if one was added

## Skill / agent instructions

Update:

- `.agents/skills/turbo-vision-4-rust/SKILL.md`
- `.github/instructions/functional-pascal.instructions.md` (TUI section)
- `.cursor/rules/functional-pascal.mdc` (`Std.Tui` table)
