# Implementation status

Living progress log for branch `refactor/tui-try-2`. Update each work session.

## Current phase

**Phase 1** — session integration landed (`open_try2_session` on Open/OpenForTest).  
**Phase 2** — in progress (headless `ExecView` works via `HeadlessTvApp`; intrinsics not wired yet).

## Landed (2026-07-07)

| Item | Location |
| --- | --- |
| Baseline snapshot | [baseline.md](baseline.md) |
| `ViewRegistry` + tests | `crates/fpas-vm/.../try2/registry.rs` |
| Rect conversion | `try2/geometry.rs` |
| `CM_*` constants (internal) | `crates/fpas-std/src/tui/cm_constants.rs` |
| `Try2Session` on `Worker` | `try2/session.rs`, `worker.rs` |
| Lifecycle hooks (reset on close) | `lifecycle.rs` |
| `open_try2_session` on Open / OpenForTest | `lifecycle.rs`, `application.rs`, `testing.rs` |
| `Dialog.NewModal` (Rust internal) | `try2/views/dialog.rs` |
| `Dialog.Add(Button)` (Rust internal) | `try2/views/button.rs` |
| Headless `ExecView` (Rust internal) | `try2/modals.rs`, `try2/headless.rs`, `headless_tv_draw.rs` |
| Live `ExecView` (interactive, no try-1 populate) | `try2/app.rs`, `try2/modals.rs` |

## Tests passing

```bash
cargo test -p fpas-vm try2::
```

Covers: registry, geometry, session open/reset, dialog new, button add, **headless ExecView → CM_OK**.

## Blockers

### Upstream `Application::with_terminal` (optional)

Headless try-2 modals now run through `HeadlessTvApp::exec_modal_view` (desktop + headless backend), not full `turbo_vision::app::Application`. Interactive path uses `Application::new()` directly without try-1 snapshot populate.

A future upstream `Application::with_terminal` would let try-2 share one code path with interactive mode; not required for the phase-2 vertical slice.

## Next steps

1. Wire try-2 intrinsics (`TuiTry2ApplicationNew`, `TuiDialogNewModal`, …) + sema symbols
2. FPAS smoke test `tests/tui/smoke/modal_button_test.fpas`
3. `Test.InjectEvent` / `Test.Click` helpers for try-2 headless path
4. Complete phase 1: sema + compiler stubs for `Application.New` / `Close`

## Unchanged (try-1 still authoritative)

- All `Application.Create*` Pascal API
- `TurboVisionObject` snapshot + reconcile
- `tests/tui/controls/*` (37 files)
