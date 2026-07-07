# Implementation status

Living progress log for branch `refactor/tui-try-2`. Update each work session.

## Current phase

**Phase 1** — session integration landed (`open_try2_session` on Open/OpenForTest).  
**Phase 2** — intrinsics wired; FPAS smoke test passes.

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
| Pascal API + intrinsics | `Dialog.NewModal`, `Dialog.AddButton`, `Application.ExecView`, `CM_*` |
| FPAS smoke test | `tests/tui/smoke/modal_button_try2_test.fpas` |

## Tests passing

```bash
cargo test -p fpas-vm try2::
fpas test tests/tui/smoke/modal_button_try2_test.fpas
```

Covers: registry, geometry, session, dialog, button, headless ExecView → CM_OK, end-to-end FPAS smoke.

## Blockers

### Upstream `Application::with_terminal` (optional)

Headless try-2 modals run through `HeadlessTvApp::exec_modal_view`. Interactive path uses `Application::new()` without try-1 snapshot populate.

## Next steps

1. Replace `Application.Try2InjectKeyboard` with `Test.Click` / `Test.InjectEvent`
2. `Button.New` + `Dialog.Add` overloads (replace `Dialog.AddButton`)
3. `Application.Run` + `OnCommand` without offset translation
4. Complete phase 1: sema stubs for `Application.New` alias

## Unchanged (try-1 still authoritative)

- All `Application.Create*` Pascal API
- `TurboVisionObject` snapshot + reconcile
- `tests/tui/controls/*` (37 files)
