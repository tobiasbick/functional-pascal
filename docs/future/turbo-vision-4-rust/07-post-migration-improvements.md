# Post-Migration Improvements

Status: **Phases A–G complete** on branch `turbo-vision-4-rust` (last updated 2026-07-02). Core migration
(Phases 0–8 in [implementation phases](04-implementation-phases.md)) is done. This file tracks what landed
after migration and what is still open.

Current user-facing behavior lives in `docs/pascal/std/tui/`. Do not describe unimplemented APIs there.

## Summary

The Turbo Vision facade is usable for real apps: modal `ExecDialog` with `InputText` and `Checked`
read-back, multi-item menus, live widget tree reconcile, runtime property setters, chrome refresh,
command-id collision guard, scripted interactive-loop tests, and optional `OnKey`/`OnMouse` hooks.

## Remaining work

Implement **one item per change**. Verify with `cargo fmt`, `cargo build`, `cargo test --workspace`, and
`cargo run -p fpas-cli -- test tests/` when `.fpas` tests change.

| Priority | Item | Status | Notes |
| --- | --- | --- | --- |
| 1 | **RadioButton read-back after `ExecDialog`** | not started | Add `Application.Selected(App, RadioButton): boolean`, mirroring `Application.Checked`: shared bool cell, `BridgedRadioButton` in modal dialogs, group exclusivity synced on close. Copy from `Checked` / `bridged_check_box.rs`. |
| 2 | **Headless paint — full menu/status chrome** | partial | `headless_paint.rs` draws only the first menu title and first status item. Extend to all items so `QueryScreenCell` matches terminal layout in tests. |
| 3 | **ListBox selection read-back** | blocked | `turbo-vision` 1.3.1 has no public selected-index getter. Do not add `Application.ListSelection` until upstream exposes one or FPAS owns selection state explicitly. |
| 4 | **Remove hosted canvas loop** | deferred | `Application.Configure` + internal `Host*` intrinsics still power `examples/pascal/tui/minimal_application.fpas` and graph demos. Separate product decision; document in `docs/pascal/` only after removal. |
| 5 | **Manual terminal verification** | open | Checklist in [testing plan](05-testing-plan.md) and `docs/pascal/std/tui/terminal-checklist.md`. |
| 6 | **Merge `turbo-vision-4-rust`** | open | Repository decision when manual checks and remaining items are acceptable. |

### Known limits (documented, not necessarily bugs)

- Live desktop **rebuild** resets focus and re-seeds widgets from FPAS state; uncommitted edits in a
  non-modal dialog are lost — use `Application.ExecDialog` for modal read-back.
- Headless `ExecDialog` does not run Turbo Vision views; use `TestSetDialogResult` and seed state via
  `SetChecked` / `SetText` for automated tests.
- Mixing `Application.Configure` (hosted canvas) with Turbo Vision `Create*` handles in one app is
  unsupported.

## Phase history

| Phase | Topic | Status |
| --- | --- | --- |
| A | Modal `ExecDialog`, `InputText`, `DialogResult`, `TestSetDialogResult` | landed |
| B | Multi-item `Menu` / `MenuItem`, separators | landed |
| C | Live widget tree reconcile during `Run` | landed |
| D | Dual-architecture docs (Turbo Vision vs hosted canvas) | landed |
| E | Command-id collision guard (`command_map.rs`) | landed |
| F | Interactive-loop testability seam (`interactive_loop.rs`) | landed |
| G | Optional `OnKey` / `OnMouse` Turbo Vision hooks | landed |
| — | Runtime setters (`SetText`, `SetChecked`, `SetItems`, `SetTitle`, `SetMenus`, `SetStatusItems`) | landed |
| — | Chrome sync on reconcile (`turbo_vision_sync_chrome_from_fpas`) | landed |
| — | `Application.Checked` + modal checkbox bridge | landed |
| — | Screen query bounds validation | landed |

## Landed detail (by theme)

### Command routing and loop

- `tv_run.rs` — custom interactive loop; FPAS `OnCommand` for unhandled Turbo Vision commands.
- `interactive_loop.rs` — `TurboVisionInteractiveSession` for scripted Rust tests without a terminal.
- `tv_input_events.rs` — `Application.OnKey`, `Application.OnMouse`.

### Modal read-back

- `exec_dialog.rs` — `ExecDialog`, `InputText`, `Checked`, `TestSetDialogResult`.
- `turbo_vision_input_text_cell.rs` — shared `InputLine` text after modal `execute`.
- `turbo_vision_bool_cell.rs`, `bridged_check_box.rs` — shared checkbox state in modal dialogs.
- Tests: `tui_turbo_vision_exec_dialog_test.fpas`, `tui_turbo_vision_checked_test.fpas`.
- Example: `examples/pascal/tui/exec_dialog.fpas` (name + newsletter checkbox).

### Live tree and chrome

- `reconcile.rs`, `tv_run.rs` — full desktop rebuild when `pending_reconcile` is set after any handled event.
- `turbo_vision_sync_chrome_from_fpas` — menu bar and status line refreshed on rebuild.
- Runtime setters mark dirty and re-render (bytecode `458`–`463`).
- Tests: `tui_turbo_vision_live_tree_test.fpas`, `tui_turbo_vision_live_dialog_test.fpas`, `tui_turbo_vision_set_*_test.fpas`.
- Example: `examples/pascal/tui/runtime_setters.fpas`.

### Safety and queries

- `command_map.rs` — reserved Turbo Vision command ids offset for user widgets.
- `query_host.rs` — `QueryScreenLine` / `QueryScreenCell` validate against painted screen size.

## How to work in this repo

- Read [`AGENTS.md`](../../../AGENTS.md) and
  [`.agents/skills/fpas-change-checklist/SKILL.md`](../../../.agents/skills/fpas-change-checklist/SKILL.md).
- One concern per file; keep files under ~400–500 LOC.
- Verify (repo root):

  ```text
  cargo fmt
  cargo build
  cargo test --workspace
  cargo run -p fpas-cli -- test tests/
  cargo run -p fpas-cli -- fmt --check tests/ examples/ apps/
  ```

## Reference recipe: add one `Std.Tui` call end to end

Copy an existing call and rename. Good anchors:

- **Widget/create** → `Application.CreateInputLine`
- **Modal / read-back** → `Application.InputText` or `Application.Checked`
- **Runtime mutation** → `Application.SetText`

| # | Layer | File |
| --- | --- | --- |
| 1 | Symbol | `crates/fpas-std/src/std_units/symbols/std_symbols.rs` |
| 2 | Bytecode | `crates/fpas-bytecode/src/intrinsic/tui/variants/widgets.inc` (next free id after highest in file; currently `Checked = 464`) |
| 3 | Sema | `crates/fpas-sema/src/std_registry/loaded/tui/application_api.rs` (+ `builtins/tui.rs` if polymorphic) |
| 4 | Compiler | `crates/fpas-compiler/src/compiler/std_calls/tui/application.rs` |
| 5 | VM | `crates/fpas-vm/src/vm/execute/io/tui/` + dispatch in `mod.rs` |
| 6 | Docs + tests | `docs/pascal/std/tui/app/`, `tests/tui/controls/*_test.fpas` |

VM notes:

- Headless: `tui.session.is_headless()` — queue test values like `TestSetDialogResult`.
- Modal/run calls: reject non-main task (`current_task_id != 0`).
- Pop stack args in reverse of Pascal parameter order.

## Upstream facts (`turbo-vision` 1.3.1)

Re-verify if the crate version changes (`~/.cargo/registry/src/*/turbo-vision-1.3.1/`).

- `Dialog::execute` — modal close command; basis for `Application.ExecDialog`.
- `InputLine::new(..., Rc<RefCell<String>>)` — FPAS mirrors via `TurboVisionInputTextCell`.
- `CheckBox::is_checked` — no shared cell upstream; FPAS uses `BridgedCheckBox` + `TurboVisionBoolCell` for modal sync.
- `ListBox` — no public selection getter in 1.3.1.
- `Application::run` — handles only built-in commands; FPAS owns the interactive loop in `tv_run.rs`.
