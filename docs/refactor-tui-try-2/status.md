# Implementation status

Living progress log for branch `refactor/tui-try-2`. Update each work session. Plan docs last synced with code: **2026-07-08**.

## Current phase

**Phase 1** — **complete** (foundation; `TuiState` slimming deferred to phase 7).  
**Phase 2** — **complete** (vertical slice + interactive manual smoke verified).  
**Phase 3** — **complete** (run loop, desktop/window, static text, chrome).
**Phase 4** — **complete** (all phase-1 widgets on try-2 path; old control tests cleanup pending).
**Phase 5** — **partial** (`MessageBox`, `OnKey`, `OnMouse` landed; `RunFileDialog` live path landed, headless still uses try-1 queued result).
**Phase 6** — **partial** (`apps/ide` source/tests migrated for menu, status, About, Open, Exit; manual terminal sign-off pending).

## Phase 1 closure notes

| Item | Resolution |
| --- | --- |
| `Try2Session` + `ViewRegistry` | Done — `Worker.try2`, lifecycle hooks, registry tests. |
| `Application.New` / `Close` | Done — `New` → `ApplicationOpen`; `Close` / `CloseForTest` + `try2.reset()`. |
| Slim `TuiState` | **Not phase 1** — needs try-1 deletion (phase 7); try-1 `tests/tui/controls/*` still authoritative. |

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
| Pascal API + intrinsics | `Dialog.NewModal`, `Button.New`, `Dialog.Add`, `Dialog.AddButton`, `Application.ExecView`, `CM_*` |
| FPAS smoke test | `tests/tui/smoke/modal_button_try2_test.fpas` (`Button.New` + `Dialog.Add`) |
| `Application.Run` (try-2 path) | `try2/run.rs`; routes when try-2 session open and no try-1 widgets |
| `OnCommand` without offset | `try2/events.rs` |
| `Application.Try2InjectCommand` | headless command injection for run tests |
| FPAS run smoke test | `tests/tui/smoke/run_quit_try2_test.fpas` |
| `Application.TestClickButton` (try-2 headless mouse) | `try2/testing.rs` |
| `Application.New` alias | sema + compiler → `ApplicationOpen` |
| `Window.New`, `Window.Add`, `Desktop.Add` | `try2/views/window.rs`, `try2/views/desktop.rs`; intrinsics 480–482 |
| FPAS window + quit smoke test | `tests/tui/smoke/window_quit_try2_test.fpas` |
| `StaticText.New`, `Window.Add` / `Dialog.Add` child dispatch | `try2/views/static_text.rs`, `try2/views/attach.rs` |
| `MenuBar.New`, `StatusLine.New`, `SetMenuBar` / `SetStatusLine` | `try2/chrome.rs`; routes when try-2 handles run |
| `Application.Run(App, OnCommand)` | sema `builtins/tui.rs`; intrinsic `ApplicationRunWithOnCommand` (484) |
| FPAS chrome + run smoke tests | `window_chrome_try2_test.fpas`, `run_quit_try2_test.fpas` |
| Try-2 window example | `examples/pascal/tui/turbo_vision_window_try2.fpas` |
| Phase-4 controls | `InputLine`, `ListBox`, `CheckBox`, `RadioButton`, `Memo`, `TextViewer` in `try2/views/` |
| Phase-4 control tests | `tests/tui/views/*_try2_test.fpas` |
| Phase-5 modal/helper routes | `try2/message_box.rs`, `try2/file_dialog.rs`, `Application.OnKey`, `Application.OnMouse` |
| Phase-5 tests | `tests/tui/modals/message_box_try2_test.fpas`, `tests/tui/events/on_key_try2_test.fpas` |

## Landed (2026-07-08)

| Item | Location |
| --- | --- |
| IDE command constants | `CM_OPEN`, `CM_ABOUT`, `CM_USER` exported through `Std.Tui`, sema, and compiler built-in consts |
| Try-2 message box example | `examples/pascal/tui/message_box.fpas` now uses `Application.New` and `CM_OK` |
| Try-2 file dialog example | `examples/pascal/tui/file_dialog_try2.fpas` |
| IDE menu/status shell migration | `apps/ide/src/menu.fpas`, `apps/ide/src/shell.fpas`, `apps/ide/src/dialog/about.fpas` |
| IDE try-2 tests | `apps/ide/tests/` uses `Application.Run(App, OnCommand)`, `Try2InjectCommand`, and `Try2InjectKeyboard` |

```bash
cargo test -p fpas-vm try2::
fpas test tests/tui/smoke/
fpas test tests/tui/views/
fpas test tests/tui/modals/message_box_try2_test.fpas
fpas test tests/tui/events/on_key_try2_test.fpas
fpas test apps/ide/tests/
cargo test -p fpas-cli fpas_regression_suite_passes
```

Covers: registry, geometry, session, dialog, button, window, desktop, static text, chrome, phase-1 widgets, headless ExecView → CM_OK, TestClickButton, `Run(App, OnCommand)`, message box, `OnKey`, IDE menu/status/dialog flows, run/quit + window/quit + window/chrome smoke; full regression suite (try-1 + try-2 coexistence).

## Next steps

1. **Finish Phase 5 blocker** — replace the headless `RunFileDialog` try-1 queued result path when upstream exposes a headless-safe picker API or an acceptable local adapter is designed.
2. **Finish Phase 6** — run the manual terminal checklist for IDE menus, About, Open file, and Exit.
3. **Phase 7** — delete try-1 bridge, rewrite the public `docs/pascal/std/tui/` spec, and archive this plan directory.

## Blockers

### Upstream `Application::with_terminal` (optional)

Headless try-2 modals run through `HeadlessTvApp::exec_modal_view`. Interactive path uses `Application::new()` without try-1 snapshot populate.

### Headless `FileDialog::execute`

Upstream `FileDialog::execute(&mut Application)` is available for live `Application`, but the branch cannot currently construct a full upstream `Application` over the headless terminal because the required fields/constructors are private. The current headless `RunFileDialog` test path therefore still uses the try-1 `test_file_dialog_result` queue instead of duplicating upstream picker logic.

## Unchanged (try-1 still authoritative)

- All `Application.Create*` Pascal API
- `TurboVisionObject` snapshot + reconcile
- `tests/tui/controls/*` (37 files)
