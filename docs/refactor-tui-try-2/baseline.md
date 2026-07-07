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

## What try-2 changes first (phase 1)

Internal-only additions on this branch (try-1 API still live):

```text
crates/fpas-vm/src/vm/execute/io/tui/try2/
  mod.rs
  registry.rs    — FpasViewId ↔ upstream ViewId
  geometry.rs    — Rect conversion (extracted from tv_geometry.rs)

crates/fpas-std/src/tui/cm_constants.rs — upstream CM_* values for try-2 Pascal surface
```

No Pascal-visible API switch until phase 2 vertical slice.

## Verification commands at baseline

All pass on branch tip before phase 1 code:

```bash
cargo build --workspace
cargo test --workspace
fpas test tests/tui/
fpas test apps/ide/tests/
```

Record results when phase 1 lands; try-1 suite must keep passing until deliberate break in phase 2.
