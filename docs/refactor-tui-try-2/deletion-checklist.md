# Deletion checklist

Audited status of the Phase-7 deletion work. The root bridge is gone; this file records the intentional exceptions and commands for re-checking the final upstream-dependent cleanup.

## Pascal public API (sema / docs)

### Removed Try-1 symbols (confirmed)

- `Application.CreateDialog`, `CreateWindow`, `CreateButton`, `CreateStaticText`
- `Application.CreateMemo`, `CreateTextViewer`, `CreateInputLine`
- `Application.CreateListBox`, `CreateOutline`, `CreateCheckBox`, `CreateRadioButton`
- `Application.CreateMenuBar`, `CreateStatusLine`
- `Application.AddChild`, `AddWindow`
- `Application.ExecDialog` (replaced by `ExecView`)
- `Application.InputText`, `Checked`, `Selected`
- `Application.ListSelection`, `OutlineSelection`, `OutlineSelectedText`
- `Application.SetText`, `SetChecked`, `SetItems`, `SetOutlineNodes`, `SetTitle`
- `Application.SetMenus`, `SetStatusItems` → move to `MenuBar.SetMenus`, `StatusLine.SetItems`
- `Application.Pump`
- `Command.Quit`, `Command.Close`, `Command.Accept`, `Command.Cancel`
- Legacy value types `ViewId`, `DialogResult`, `ScreenCell`, `TuiEvent`, `EventKind`, and `ExitReason`

### Current Try-2 testing surface (intentionally retained)

- `Application.TestClickButton`, `TestClickMouse`, and `TestDispatchMenuCommand`
- `Application.TestSetFileDialogResult` and `TestSetDialogResult`
- `Application.TestInjectKeyboard` and `TestInjectCommand` (interim headless event helpers)

`Application.SetMenuBar` and `Application.SetStatusLine` are current API and remain registered.

### Replaced symbols (not deleted, renamed)

| try-1 | try-2 |
| --- | --- |
| `Application.Open` | `Application.New` (both remain registered) |
| `Application.ExecDialog` | `Application.ExecView` |
| `Application.InputText` | `InputLine.Text` |
| `Application.Checked` | `CheckBox.Checked` |
| `Application.Selected` | `RadioButton.Selected` |
| `Application.ListSelection` | `ListBox.Selection` |
| `Application.AddWindow` | `Desktop.Add` |

## Bytecode intrinsics

Try-1 `TuiCreate*`, child-add, input, pump, and snapshot intrinsics have been removed. The current `try2.inc` retains `Try2InjectKeyboard` and `Try2InjectCommand` because the Pascal surface still exposes them as interim headless test helpers.

## VM modules

All former root bridge modules have been deleted or moved into `try2/`; `tui/mod.rs` is now dispatch and re-exports only. Direct upstream views replaced the `StaticText`, `Button`, `Memo`, `TextViewer`, and `ListBox` adapters.

The three remaining `try2/bridged_*.rs` files require an upstream read-back hook before removal. Do not delete them until `CheckBox`, `RadioButton`, and `OutlineViewer` expose a supported runtime state query through `dyn View`.

## VM types

`TurboVisionState`, `TurboVisionObject`, snapshot structs, reconcile queues, and old live-view id lists have been removed. Headless modal and file-dialog queues belong to `Try2Session`.

## fpas-std

- ~~`command_ids.rs`~~ — deleted; `CM_*` in `cm_constants.rs` is canonical
- Removed 59 unregistered Try-1 and retained-host symbol names from `std_symbols/tui.rs` on 2026-07-11.
- Removed six undocumented legacy value types from the FPAS symbol table, Sema registration, and compiler special cases on 2026-07-11. Internal Rust input events remain implementation detail only.

The following patterns must return **no hits** in `crates/`:

```text
TurboVisionObject
pending_reconcile
turbo_vision_reconcile
FPAS_TV_COMMAND_OFFSET
TuiCreateDialog
TuiAddChild
```

`bridged_` is intentionally limited to `try2/bridged_check_box.rs`, `try2/bridged_radio_button.rs`, and `try2/bridged_outline.rs` until upstream exposes a read-back hook.

## Tests

**Completed (2026-07-09):** all 37 try-1 control tests and `tests/tui/controls/` were removed. Try-2 regressions live under `tests/tui/views/`, `tests/tui/smoke/`, `tests/tui/events/`, and `tests/tui/modals/`.

| Removed | try-2 replacement |
| --- | --- |
| Phase-1 widgets + read-back/setter (13) | `tests/tui/views/*_try2_test.fpas` |
| `run`, `window`, `chrome`, `menu`, `exec_dialog`, `static_text` (6) | `tests/tui/smoke/*_try2_test.fpas` |
| `message_box` (1) | `tests/tui/modals/message_box_try2_test.fpas` |
| `file_dialog` (1) | `tests/tui/modals/file_dialog_try2_test.fpas` |

No try-1 control tests remain. The retained Try-2 tests keep their `_try2` suffix until the final naming cleanup.

## Examples

The TUI examples use the current Try-2 factories and modal API. No Try-1 example migration remains in this checklist.

## Documentation

The public `docs/pascal/std/tui/` pages were rewritten for the direct Try-2 API. [app/types.md](../pascal/std/tui/app/types.md) defines the complete public type surface. Keep the plan directory as the implementation handoff and upstream-adapter record; it is not a public API specification.

## Planning docs

Keep `docs/refactor-tui-try-2/` until the upstream adapter blocker is resolved. Update [status.md](status.md) and this checklist when that changes.

## Skill / agent instructions

Updated on 2026-07-11: `.agents/skills/turbo-vision-4-rust/SKILL.md`, `.github/instructions/functional-pascal.instructions.md`, and `.cursor/rules/functional-pascal.mdc` describe the direct Try-2 layout.
