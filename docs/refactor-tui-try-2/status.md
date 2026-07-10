# Implementation status

Living progress log for branch `refactor/tui-try-2`. Update each work session. Plan docs last synced with code: **2026-07-09**.

## Current phase

**Phase 1** — **complete** (foundation; `TuiState` slimming deferred to phase 7).  
**Phase 2** — **complete** (vertical slice + interactive manual smoke verified).  
**Phase 3** — **complete** (run loop, desktop/window, static text, chrome).
**Phase 4** — **complete** (all phase-1 widgets on try-2 path including `Outline`; control tests migrated in phase 7).
**Phase 5** — **complete** for current branch scope (`MessageBox`, `OnKey`, `OnMouse`, `RunFileDialog` with Try-2-local headless adapter).  
**Phase 6** — **complete** (`apps/ide` migrated; automated + manual terminal sign-off green).  
**Phase 7** — **in progress** (try-1 snapshot removed; `docs/pascal/std/tui/` rewrite remains for Phase 8).

## Phase 1 closure notes

| Item | Resolution |
| --- | --- |
| `Try2Session` + `ViewRegistry` | Done — `Worker.try2`, lifecycle hooks, registry tests. |
| `Application.New` / `Close` | Done — `New` → `ApplicationOpen`; `Close` / `CloseForTest` + `try2.reset()`. |
| Slim `TuiState` | **Next** — try-1 control tests removed; delete VM try-1 modules per [deletion-checklist.md](deletion-checklist.md). |

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
| Phase-4/7 setters | `StaticText.SetText`, `Button.SetText`, `Dialog.SetTitle`, `Window.SetTitle`, `MenuBar.SetMenus`, `StatusLine.SetItems` on try-2 path |
| Phase-5 modal/helper routes | `try2/message_box.rs`, `try2/file_dialog.rs`, `Application.OnKey`, `Application.OnMouse` |
| Phase-5 tests | `tests/tui/modals/message_box_try2_test.fpas`, `tests/tui/modals/file_dialog_try2_test.fpas`, `tests/tui/events/on_key_try2_test.fpas`, `tests/tui/events/on_mouse_try2_test.fpas` |
| Try-2 headless file dialog queue | `Try2Session::set_file_dialog_result`; `Application.TestSetFileDialogResult` seeds Try-2 state when the Try-2 session is open |
| Try-2 menu dispatch in headless tests | `Application.TestDispatchMenuCommand` routes through try-2 menu bar state |
| Menu command smoke test | `tests/tui/smoke/menu_dispatch_try2_test.fpas` |

## Landed (2026-07-08)

| Item | Location |
| --- | --- |
| IDE command constants | `CM_OPEN`, `CM_ABOUT`, `CM_USER` exported through `Std.Tui`, sema, and compiler built-in consts |
| Try-2 message box example | `examples/pascal/tui/message_box.fpas` now uses `Application.New` and `CM_OK` |
| Try-2 file dialog example | `examples/pascal/tui/file_dialog_try2.fpas` |
| IDE menu/status shell migration | `apps/ide/src/menu.fpas`, `apps/ide/src/shell.fpas`, `apps/ide/src/dialog/about.fpas` |
| IDE try-2 tests | `apps/ide/tests/` uses `Application.Run(App, OnCommand)`, `Try2InjectCommand`, and `Try2InjectKeyboard` |

## Verified (2026-07-09)

| Check | Result |
| --- | --- |
| TUI sema surface | `cargo test -p fpas-sema std_units::tui` — 17 passed |
| TUI compiler lowering/runtime tests | `cargo test -p fpas-compiler std_library::tui` — 10 passed |
| TUI Rust doc links | `cargo test -p fpas-vm tui_spec_links` — 2 passed |
| Try-1 Turbo Vision controls coexistence | `cargo run -q -p fpas-cli -- test tests/tui/controls/` — 6 passed |
| IDE automated flows | `cargo run -q -p fpas-cli -- test apps/ide/tests/` — 7 passed |
| Try-2 menu command dispatch | `cargo run -q -p fpas-cli -- test tests/tui/smoke/menu_dispatch_try2_test.fpas` — passed |
| Relevant FPAS formatting | `cargo run -q -p fpas-cli -- fmt --check tests/tui/events/on_mouse_try2_test.fpas apps/ide/tests/` — passed |

## Manual sign-off (2026-07-09)

| Check | Result |
| --- | --- |
| IDE File / Exit | Passed |
| IDE Help / About | Passed |
| IDE File / Open | Passed |
| IDE resize | Passed |

Automated coverage: IDE `about_menu_test.fpas` and `open_menu_test.fpas` call `Application.TestDispatchMenuCommand`, exercising menu bar → command id → `OnCommand` flow.

```bash
cargo test -p fpas-vm try2::
fpas test tests/tui/smoke/menu_dispatch_try2_test.fpas
fpas test tests/tui/views/
fpas test tests/tui/modals/message_box_try2_test.fpas
fpas test tests/tui/events/on_key_try2_test.fpas
fpas test tests/tui/events/on_mouse_try2_test.fpas
fpas test apps/ide/tests/
cargo test -p fpas-cli fpas_regression_suite_passes
```

Covers: registry, geometry, session, dialog, button, window, desktop, static text, chrome, phase-1 widgets, headless ExecView → CM_OK, TestClickButton, `Run(App, OnCommand)`, message box, `OnKey`, `OnMouse`, IDE menu/status/dialog flows, run/quit + window/quit + window/chrome smoke; full regression suite (try-1 + try-2 coexistence).

## Next steps

1. **Phase 7 (VM deletion)** — remove try-1 VM modules, sema symbols, and public docs per [deletion-checklist.md](deletion-checklist.md). **Follow-up:** try-2 `TestClickMouse` hit-target routing exists; checkbox/radio toggle via mouse on desktop windows needs a dedicated fix (smoke tests verify coordinate routing only).
2. **Phase 7/8 cleanup** — replace interim `TestSetFileDialogResult` naming with the final headless event API after the public testing surface is rewritten.

## Phase 7 progress (2026-07-09)

| Batch | Deleted (try-1) | try-2 replacement |
| --- | --- | --- |
| Widgets + read-back | 13 tests (`check_box`, `input_line`, `list_box`, `memo`, `radio_button`, `text_viewer`, `checked`, `list_selection`, `set_items`, `set_text_*`, `radio_selected`) | `tests/tui/views/*_try2_test.fpas` (6) |
| Run / chrome / modals | 7 tests (`run`, `window`, `chrome`, `menu`, `exec_dialog`, `message_box`, `static_text`) | `tests/tui/smoke/*_try2_test.fpas`, `tests/tui/modals/message_box_try2_test.fpas` |

| Phase-5 modal | 1 test (`file_dialog`) | `tests/tui/modals/file_dialog_try2_test.fpas` |
| Custom command id | 1 test (`reserved_command`) | `tests/tui/smoke/reserved_command_try2_test.fpas` |
| Setters / title | 3 tests (`set_text`, `set_text_button`, `set_title`) | `tests/tui/views/*_set_*_try2_test.fpas` |
| Chrome / pump | 6 tests (`set_menus`, `set_status_items`, `chrome_paint`, `set_checked`, `spike`, `tui_run_path`) | `tests/tui/smoke/*_try2_test.fpas` |

| Metric | Value |
| --- | --- |
| Controls tests removed | **37 / 37** |
| Examples migrated to try-2 | **5 / 5** (`exec_dialog`, `runtime_setters`, `turbo_vision_dialog`, `turbo_vision_outline`, `turbo_vision_window`) |
| VM / sema / `docs/pascal/std/tui/` rewrite | Not started |

Remaining Phase 7 work: remove try-1 VM modules and sema symbols per [deletion-checklist.md](deletion-checklist.md); simplify coexistence routing once try-1 intrinsics are gone.

## Blockers

### Upstream `Application::with_terminal` (optional)

Headless try-2 modals run through `HeadlessTvApp::exec_modal_view`. Interactive path uses `Application::new()` without try-1 snapshot populate.

### Headless `FileDialog::execute`

Upstream `FileDialog::execute(&mut Application)` is available for live `Application`, but the branch cannot currently construct a full upstream `Application` over the headless terminal because the required fields/constructors are private. The current headless `RunFileDialog` test path uses a Try-2-local queued adapter on `Try2Session`; it no longer consumes the try-1 `test_file_dialog_result` queue.

## Unchanged (Phase 8)

- `docs/pascal/std/tui/` — public spec rewrite in Phase 8

## Phase 7 snapshot cleanup (2026-07-10)

- `TurboVisionObject` enum and widget snapshot structs removed from `shared/tui.rs`
- `TurboVisionState` reduced to headless `test_dialog_result` stub
- Dead try-1 handle decoders removed from `handles.rs`; `input_line_view_bindings` removed from `Worker`
- `fpas-std` symbol table: dropped try-1 `Application.Create*` / `Pump` / setter getters
- `TestClickButton` and file-dialog tests no longer reference try-1 queues

## Phase 7 VM batch (2026-07-10)

Removed legacy try-1 run/reconcile bridge:

- Deleted: `tui_run.rs`, `tv_run.rs`, `reconcile.rs`, `live_patch.rs`, `interactive_loop.rs`, `tv_views.rs`, `menu_build.rs`
- `Application.Run` always uses `try2/run.rs` (no coexistence fallback)
- Bytecode: dropped try-1 widget intrinsics from `widgets.inc` (`Create*`, `Pump`, `ExecDialog`, setters/getters); kept shared chrome/modal/test intrinsics
- Headless paint via try-1 `turbo_vision_populate_desktop` removed from `headless_tv_draw.rs`

## Phase 7 VM batch (2026-07-09)

Deleted try-1-only VM modules:

- `control_create.rs`, `controls.rs`, `dialogs.rs`, `exec_dialog.rs`, `outline_read.rs`, `windows.rs`

`try_exec_turbo_vision_intrinsic` now dispatches only: `MessageBox`, `RunFileDialog`, callback registration (`OnKey`/`OnMouse`/`OnCommand`), `Quit`, chrome attach (`SetMenuBar`/`SetStatusLine`), and test helpers.
