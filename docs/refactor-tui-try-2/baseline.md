# Baseline snapshot (try-1)

Frozen reference state at the start of implementation on branch `refactor/tui-try-2`.

| Field | Value |
| --- | --- |
| Date | 2026-07-07 |
| Branch | `refactor/tui-try-2` (from `main` @ `4aa19f8f`) |
| Upstream pin | `turbo-vision` 2.0.0, git tag `v2.0.0` |
| Plan commit | `4aa19f8f` — docs in `docs/refactor-tui-try-2/` |

## Public Pascal API (try-1)

Documented in [`docs/pascal/std/tui/`](../pascal/std/tui/). Summary:

- Session: `Application.Open`, `OpenForTest`, `Close`, `Size`, `Run`, `Quit`
- Construction: `Application.Create*` (dialog, window, controls, menu, status)
- Composition: `Application.AddChild`, `AddWindow`
- Modals: `ExecDialog`, `MessageBox`, `RunFileDialog`
- Read-back: `InputText`, `Checked`, `Selected`, `ListSelection`, outline helpers
- Runtime setters: `SetText`, `SetChecked`, `SetItems`, `SetTitle`, `SetMenus`, `SetStatusItems`
- Handlers: `OnCommand`, `OnKey`, `OnMouse`
- Headless: `Pump`, `TestClickButton`, `TestClickMouse`, `TestDispatchMenuCommand`, `TestSetDialogResult`, `TestSetFileDialogResult`
- Commands: `Command.Quit` (1), `Command.Close` (4), `Command.Accept` (10), `Command.Cancel` (11) + offset band `0x8000` for collisions

## VM bridge (try-1)

| Metric | Count |
| --- | --- |
| Modules under `crates/fpas-vm/src/vm/execute/io/tui/` | 41 |
| Approximate LOC | 6,526 |
| `Bridged*` adapter views | 8 |
| Key modules | `reconcile.rs`, `live_patch.rs`, `command_map.rs`, `session_app.rs`, `tv_run.rs` |

Architecture: FPAS `TurboVisionObject` snapshot → `pending_reconcile` → full desktop rebuild on live `turbo_vision::Application`.

Worker field: `live_turbo_vision_app: Option<TurboVisionApplication>` on main worker only.

## FPAS regression tests

| Area | Count |
| --- | --- |
| `tests/tui/controls/*_test.fpas` | 37 |
| `apps/ide/tests/` | shell, menu, dialog, status, theme |

## `fpas-std` TUI runtime

- `tui/command.rs` — host `CommandRegistry` / `CommandEvent` (retained-engine leftovers + shortcuts)
- `tui/command_ids.rs` — four `COMMAND_*` constants for try-1 Pascal `Command.*`
- `tui/session/` — `TuiSession`, damage tracking
- Inline tests under `tui/tests/`, `graph/tests/`, `console/tests/`

## Upstream `test-util` (v2.0.0)

Feature flag `test-util` is empty in upstream `Cargo.toml`; module `turbo_vision::test_util` provides:

| Type | Purpose |
| --- | --- |
| `MockTerminal` | In-memory terminal buffer (width × height) |
| `MockTerminal::push_event` | Queue synthetic `Event`s |
| `MockTerminal::poll_event` | Dequeue events for test loops |
| `get_row`, `get_rect_text`, `fill_rect` | Screen assertions |

Try-2 headless path should prefer `MockTerminal` + `put_event` over FPAS `TestSetDialogResult` stubs where possible. FPAS already has `tv_headless_backend.rs` — evaluate merge vs replace in phase 2.

## Sema / compiler surface (try-1)

- Sema: `crates/fpas-sema/src/std_registry/loaded/tui/` — `application_api.rs` (~70 symbols), `command_api.rs`, `handlers.rs`, `message_box_api.rs`
- Compiler: `crates/fpas-compiler/src/compiler/std_calls/tui/`
- Bytecode: `TuiIntrinsic` enum — `TuiCreate*`, `TuiAddChild`, `TuiExecDialog`, … (~45 variants)

## IDE (`apps/ide`)

Uses try-1 API throughout `src/` (menu, shell, dialog/open, about message box). Tests in `apps/ide/tests/`.

## What try-2 adds (phase 1 complete; phase 2 in progress)

Phase 1 landed internal foundation; phase 2 added branch-only Pascal symbols (not yet in `docs/pascal/`). try-1 API and tests remain authoritative until phase 7.

**Current `try2/` tree** (see [rust-layout.md](rust-layout.md)):

```text
crates/fpas-vm/src/vm/execute/io/tui/try2/
  mod.rs, session.rs, registry.rs, geometry.rs, records.rs
  events.rs, run.rs, headless.rs, modals.rs, app.rs, intrinsics.rs, testing.rs
  views/dialog.rs, views/button.rs

crates/fpas-std/src/tui/cm_constants.rs
```

**Branch-only Pascal surface** (coexists with try-1 on `refactor/tui-try-2`):

- `Dialog.NewModal`, `Button.New`, `Dialog.Add`, `Dialog.AddButton`
- `Application.ExecView`, `CM_OK` / `CM_CANCEL` / `CM_QUIT`
- `Application.Try2InjectKeyboard`, `Application.Try2InjectCommand` (headless test helpers)
- try-2 `Application.Run` when session is open and no try-1 widgets exist

**Smoke tests:** `tests/tui/smoke/modal_button_try2_test.fpas`, `run_quit_try2_test.fpas`.

## Verification commands (branch tip)

try-1 suite must keep passing while both bridges coexist:

```bash
cargo build --workspace
cargo test --workspace
cargo test -p fpas-vm try2::
fpas test tests/tui/smoke/
fpas test tests/tui/
fpas test apps/ide/tests/
cargo test -p fpas-cli fpas_regression_suite_passes
```

Recorded 2026-07-07 after phase 1 + phase 2 vertical slice: all green.
