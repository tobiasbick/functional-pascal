# Implementation status

Living progress log for branch `refactor/tui-try-2`. Update each work session. Plan docs last synced with code: **2026-07-11**.

## Current phase

**Phase 1** — **complete** (foundation; `TuiState` slimming deferred to phase 7).  
**Phase 2** — **complete** (vertical slice + interactive manual smoke verified).  
**Phase 3** — **complete** (run loop, desktop/window, static text, chrome).
**Phase 4** — **complete** (all phase-1 widgets on try-2 path including `Outline`; control tests migrated in phase 7).
**Phase 5** — **complete** for current branch scope (`MessageBox`, `OnKey`, `OnMouse`, `RunFileDialog` with Try-2-local headless adapter).  
**Phase 6** — **complete** (`apps/ide` migrated; automated + manual terminal sign-off green).  
**Phase 7** — **in progress** (bridge migration complete; three upstream read-back adapters + interim test API remain — [remaining-work.md](remaining-work.md)).
**Phase 8** — optional follow-ups; public docs were rewritten as Phase 7 work.

## Phase 1 closure notes

| Item | Resolution |
| --- | --- |
| `Try2Session` + `ViewRegistry` | Done — `Worker.try2`, lifecycle hooks, registry tests. |
| `Application.New` / `Close` | Done — `New` → `ApplicationOpen`; `Close` / `CloseForTest` + `try2.reset()`. |
| Slim `TuiState` | Done — `TurboVisionState`, `TurboVisionObject`, and snapshot structs removed. |

## Landed (2026-07-07)

| Item | Location |
| --- | --- |
| Baseline snapshot | [baseline.md](baseline.md) |
| `ViewRegistry` + tests | `crates/fpas-vm/.../try2/registry.rs` |
| Rect conversion | `tv_geometry.rs` |
| `CM_*` constants (internal) | `crates/fpas-std/src/tui/cm_constants.rs` |
| `Try2Session` on `Worker` | `try2/session.rs`, `worker.rs` |
| Lifecycle hooks (reset on close) | `lifecycle.rs` |
| `open_try2_session` on Open / OpenForTest | `lifecycle.rs`, `application.rs`, `testing.rs` |
| `Dialog.NewModal` (Rust internal) | `try2/views/dialog.rs` |
| `Dialog.Add(Button)` (Rust internal) | `try2/views/button.rs` |
| Headless `ExecView` (Rust internal) | `try2/modals.rs`, `try2/headless.rs`, `headless_tv_draw.rs` |
| Live `ExecView` (interactive, no try-1 populate) | `try2/app.rs`, `try2/modals.rs` |
| Pascal API + intrinsics | `Dialog.NewModal`, `Button.New`, `Dialog.Add`, `Application.ExecView`, `CM_*` |
| FPAS smoke test | `tests/tui/smoke/modal_button_test.fpas` (`Button.New` + `Dialog.Add`) |
| `Application.Run` (try-2 path) | `try2/run.rs`; routes when try-2 session open and no try-1 widgets |
| `OnCommand` without offset | `try2/events.rs` |
| `Application.TestInjectCommand` | headless command injection for run tests |
| FPAS run smoke test | `tests/tui/smoke/run_quit_test.fpas` |
| `Application.TestClickButton` (try-2 headless mouse) | `try2/testing.rs` |
| `Application.New` alias | sema + compiler → `ApplicationOpen` |
| `Window.New`, `Window.Add`, `Desktop.Add` | `try2/views/window.rs`, `try2/views/desktop.rs`; intrinsics 480–482 |
| FPAS window + quit smoke test | `tests/tui/smoke/window_quit_test.fpas` |
| `StaticText.New`, `Window.Add` / `Dialog.Add` child dispatch | `try2/views/static_text.rs`, `try2/views/attach.rs` |
| `MenuBar.New`, `StatusLine.New`, `SetMenuBar` / `SetStatusLine` | `try2/chrome.rs`; routes when try-2 handles run |
| `Application.Run(App, OnCommand)` | sema `builtins/tui.rs`; intrinsic `ApplicationRunWithOnCommand` (484) |
| FPAS chrome + run smoke tests | `window_chrome_test.fpas`, `run_quit_test.fpas` |
| Try-2 window example | `examples/pascal/tui/turbo_vision_window_try2.fpas` |
| Phase-4 controls | `InputLine`, `ListBox`, `CheckBox`, `RadioButton`, `Memo`, `TextViewer` in `try2/views/` |
| Phase-4 control tests | `tests/tui/views/*_test.fpas` |
| Phase-4/7 setters | `StaticText.SetText`, `Button.SetText`, `Dialog.SetTitle`, `Window.SetTitle`, `MenuBar.SetMenus`, `StatusLine.SetItems` on try-2 path |
| Phase-5 modal/helper routes | `try2/message_box.rs`, `try2/file_dialog.rs`, `Application.OnKey`, `Application.OnMouse` |
| Phase-5 tests | `tests/tui/modals/message_box_test.fpas`, `tests/tui/modals/file_dialog_test.fpas`, `tests/tui/events/on_key_test.fpas`, `tests/tui/events/on_mouse_test.fpas` |
| Try-2 headless file dialog queue | `Try2Session::set_file_dialog_result`; `Application.TestSetFileDialogResult` seeds Try-2 state when the Try-2 session is open |
| Try-2 menu dispatch in headless tests | `Application.TestDispatchMenuCommand` routes through try-2 menu bar state |
| Menu command smoke test | `tests/tui/smoke/menu_dispatch_test.fpas` |

## Landed (2026-07-08)

| Item | Location |
| --- | --- |
| IDE command constants | `CM_OPEN`, `CM_ABOUT`, `CM_USER` exported through `Std.Tui`, sema, and compiler built-in consts |
| Try-2 message box example | `examples/pascal/tui/message_box.fpas` now uses `Application.New` and `CM_OK` |
| Try-2 file dialog example | `examples/pascal/tui/file_dialog_try2.fpas` |
| IDE menu/status shell migration | `apps/ide/src/menu.fpas`, `apps/ide/src/shell.fpas`, `apps/ide/src/dialog/about.fpas` |
| IDE try-2 tests | `apps/ide/tests/` uses `Application.Run(App, OnCommand)`, `TestInjectCommand`, and `TestInjectKeyboard` |

## Verified (2026-07-09)

| Check | Result |
| --- | --- |
| TUI sema surface | `cargo test -p fpas-sema std_units::tui` — 17 passed |
| TUI compiler lowering/runtime tests | `cargo test -p fpas-compiler std_library::tui` — 10 passed |
| TUI Rust doc links | `cargo test -p fpas-vm tui_spec_links` — 2 passed |
| Try-1 Turbo Vision controls coexistence | `cargo run -q -p fpas-cli -- test tests/tui/controls/` — 6 passed |
| IDE automated flows | `cargo run -q -p fpas-cli -- test apps/ide/tests/` — 7 passed |
| Try-2 menu command dispatch | `cargo run -q -p fpas-cli -- test tests/tui/smoke/menu_dispatch_test.fpas` — passed |
| Relevant FPAS formatting | `cargo run -q -p fpas-cli -- fmt --check tests/tui/events/on_mouse_test.fpas apps/ide/tests/` — passed |

## Manual sign-off (2026-07-09)

| Check | Result |
| --- | --- |
| IDE File / Exit | Passed |
| IDE Help / About | Passed |
| IDE File / Open | Passed |
| IDE resize | Passed |

Automated coverage: IDE `about_menu_test.fpas` and `open_menu_test.fpas` call `Test.DispatchMenu`, exercising menu bar → command id → `OnCommand` flow.

```bash
cargo test -p fpas-vm try2::
fpas test tests/tui/smoke/menu_dispatch_test.fpas
fpas test tests/tui/views/
fpas test tests/tui/modals/message_box_test.fpas
fpas test tests/tui/events/on_key_test.fpas
fpas test tests/tui/events/on_mouse_test.fpas
fpas test apps/ide/tests/
cargo test -p fpas-cli fpas_regression_suite_passes
```

Covers: registry, geometry, session, dialog, button, window, desktop, static text, chrome, phase-1 widgets, headless ExecView → CM_OK, TestClickButton, `Run(App, OnCommand)`, message box, `OnKey`, `OnMouse`, IDE menu/status/dialog flows, run/quit + window/quit + window/chrome smoke; full regression suite (try-1 + try-2 coexistence).

## Next steps

See [remaining-work.md](remaining-work.md) for the ordered backlog. Summary:

1. **Stream A (blocked)** — remove `try2/bridged_{check_box,radio_button,outline}.rs` when upstream exposes live read-back.
2. **Stream B (in progress)** — `Test.Click`, `Test.DispatchMenu`, `Test.InjectCommand`, and `Test.InjectKeyboard` landed; remove interim `Application.Test*` names when ready.
3. ~~**Stream C** — rename `*_try2_test.fpas` files.~~ **Done (2026-07-11).**
4. **Stream D** — run [verification.md](verification.md) and archive this plan directory.

# Phase 7 progress (2026-07-10)

- Deleted duplicate `command_ids.rs`; `CM_*` values come from `cm_constants.rs` only
- Deleted test-only `command_map.rs`; the upstream bump checklist now compares `cm_constants.rs` directly
- Moved `commands.rs` into `try2/commands.rs`; callback, quit, and test dispatch ownership no longer lives at the legacy bridge root
- Moved `tv_geometry.rs` into `try2/geometry.rs`; chrome now owns the current rectangle conversion path
- Moved `navigation.rs` into `try2/chrome_input.rs`; chrome record decoding no longer lives at the legacy bridge root
- Moved `records.rs` into `try2/application_records.rs`; application and size value construction now lives with the Try-2 session
- Moved `handle_records.rs` into `try2/handle_records.rs`; opaque widget records now belong to the Try-2 bridge
- Moved `handles.rs` and `tv_input_events.rs` into `try2/`; widget decoding and unhandled input dispatch no longer live at the legacy bridge root
- Moved application, lifecycle, and live-session ownership into `try2/`; the legacy bridge root now retains only headless support and adapter views
- Moved test-session lifecycle and the headless backend into `try2/`; the renderer is the remaining legacy headless root module
- Deleted `BridgedStaticText`; `StaticText.SetText` replaces the upstream child view and updates its registry view id
- Deleted `BridgedButton`; `Button.SetText` uses the same direct upstream child replacement path
- Deleted `BridgedMemo` and `BridgedTextViewer`; both `SetText` routes replace the upstream child view and retain the FPAS handle
- Deleted `BridgedListBox`; direct upstream `ListBox` synchronizes selection on read-back and replaces items in place
- The remaining checkbox, radio-button, and outline adapters live under `try2/`; their host-cell synchronization remains unchanged
- Moved the headless renderer, chrome layout, and remaining adapters into `try2/`; `tui/mod.rs` is now dispatch and re-exports only
- Merged `msgbox.rs`, `file_dialog.rs`, and `test_mouse.rs` into `try2/message_box.rs`, `try2/file_dialog.rs`, and `try2/testing.rs`

## Phase 7 adapter checkpoint (2026-07-10)

- Direct upstream migration is complete for `Button`, `StaticText`, `Memo`, `TextViewer`, and `ListBox`.
- `CheckBox`, `RadioButton`, and `OutlineViewer` at the pinned `turbo-vision` `v2.0.0` revision do not implement `View::as_any_mut`.
- Their adapters remain necessary to copy live keyboard and mouse selection back to FPAS host cells. Removing them without an upstream hook would regress `Checked`, `Selected`, `Outline.Selection`, and `Outline.SelectedText` after interactive input.
- Resume this cleanup after upstream adds downcast support (or another public read-back hook) for those three view types; then replace the views directly and remove `try2/bridged_{check_box,radio_button,outline}.rs`.
- Verification at this checkpoint: focused Rust Try-2 suite (61), Sema TUI tests (18), compiler TUI tests (11), FPAS `tests/tui/` (30), and `apps/ide/tests/` (7) pass.

## Phase 7 headless modal queue fix (2026-07-11)

- `HeadlessTvApp::exec_modal_view` now preserves queued application commands while it drains modal keyboard or mouse input.
- A modal checks its closing command before polling the next queued event, so a following `CM_QUIT` reaches the outer `Application.Run` loop.
- Rust regression: `modal_preserves_queued_application_command_for_outer_run_loop`.
- IDE regressions fixed: `apps/ide/tests/shell/about_menu_test.fpas` and `open_menu_test.fpas`.

## Phase 7 instruction sync (2026-07-11)

- Updated `AGENTS.md`, `.agents/skills/turbo-vision-4-rust/SKILL.md`, `.github/instructions/functional-pascal.instructions.md`, and `.cursor/rules/functional-pascal.mdc` for the direct Try-2 API and current module layout.
- Removed stale guidance for `Application.Create*`, `AddChild`, `Pump`, `ExecDialog`, retained hosted TUI dispatch, and deleted root bridge modules.

## Phase 7 Stream C test rename (2026-07-11)

- Renamed 30 `tests/tui/**/*_try2_test.fpas` files to `*_test.fpas` (smoke, views, modals, events).
- Dropped `Try2` from program identifiers (`RunQuitTry2Test` → `RunQuitTest`, etc.).
- Updated doc paths under `docs/refactor-tui-try-2/` and `docs/pascal/std/`.

## Phase 7 Test.Click alias (2026-07-11)

- Registered `Std.Tui.Test.Click` in sema, symbols, and compiler (maps to `TestClickButton` intrinsic).
- Public docs show `Test.Click` as preferred name; `Application.TestClickButton` remains interim alias.

## Phase 7 Test.InjectCommand / Test.InjectKeyboard aliases (2026-07-11)

- Registered `Std.Tui.Test.InjectCommand` and `Std.Tui.Test.InjectKeyboard` (map to `Try2InjectCommand` / `Try2InjectKeyboard` intrinsics).
- Migrated all FPAS call sites from interim `Application.TestInject*` names to `Test.Inject*`.
- Interim `Application.TestInject*` names remain registered until Stream B step 5.

## Phase 7 Test.DispatchMenu alias + test migration (2026-07-11)

- Registered `Std.Tui.Test.DispatchMenu` (maps to `TestDispatchMenuCommand` intrinsic).
- Migrated `tests/tui/smoke/*` button tests and IDE menu tests from interim `Application.TestClickButton` / `Application.TestDispatchMenuCommand` to `Test.Click` / `Test.DispatchMenu`.
- Interim `Application.Test*` names remain registered until Stream B step 4.

## Phase 7 symbol and documentation audit (2026-07-11)

- Removed 59 unregistered Try-1 and retained-host names from `fpas-std`'s `Std.Tui` symbol table; the remaining entries match the current Sema registrations and headless test helpers.
- Removed six undocumented legacy value types (`ViewId`, `DialogResult`, `ScreenCell`, `TuiEvent`, `EventKind`, and `ExitReason`) from the FPAS surface, including compiler enum and equality special cases.
- Confirmed that `crates/fpas-vm/src/vm/execute/io/tui/` contains only `mod.rs` and `try2/`; no root Try-1 bridge module remains.
- Updated `docs/pascal/std/testing/test.md` to describe rendered Try-2 headless paths and link to the current `tests/tui/smoke/` examples.
- Rewrote [deletion-checklist.md](deletion-checklist.md) as a current audit: root migration work is complete and only the three upstream read-back adapters remain.
- Added [remaining-work.md](remaining-work.md) — ordered backlog for adapter removal, test API, rename, and plan archive.

## Next work item

**Stream B (in progress):** `Test.Click`, `Test.DispatchMenu`, `Test.InjectCommand`, and `Test.InjectKeyboard` registered; FPAS tests migrated (2026-07-11) — see [remaining-work.md](remaining-work.md).

**Stream A (blocked):** remove `CheckBox`, `RadioButton`, and `Outline` adapters when the pinned upstream version provides a read-back hook. Do not delete the plan directory until verification is green.

## Phase 7 progress (2026-07-09)

| Batch | Deleted (try-1) | try-2 replacement |
| --- | --- | --- |
| Widgets + read-back | 13 tests (`check_box`, `input_line`, `list_box`, `memo`, `radio_button`, `text_viewer`, `checked`, `list_selection`, `set_items`, `set_text_*`, `radio_selected`) | `tests/tui/views/*_test.fpas` (6) |
| Run / chrome / modals | 7 tests (`run`, `window`, `chrome`, `menu`, `exec_dialog`, `message_box`, `static_text`) | `tests/tui/smoke/*_test.fpas`, `tests/tui/modals/message_box_test.fpas` |

| Phase-5 modal | 1 test (`file_dialog`) | `tests/tui/modals/file_dialog_test.fpas` |
| Custom command id | 1 test (`reserved_command`) | `tests/tui/smoke/reserved_command_test.fpas` |
| Setters / title | 3 tests (`set_text`, `set_text_button`, `set_title`) | `tests/tui/views/*_set_*_test.fpas` |
| Chrome / pump | 6 tests (`set_menus`, `set_status_items`, `chrome_paint`, `set_checked`, `spike`, `tui_run_path`) | `tests/tui/smoke/*_test.fpas` |

| Metric | Value |
| --- | --- |
| Controls tests removed | **37 / 37** |
| Examples migrated to try-2 | **5 / 5** (`exec_dialog`, `runtime_setters`, `turbo_vision_dialog`, `turbo_vision_outline`, `turbo_vision_window`) |
| VM / sema / `docs/pascal/std/tui/` rewrite | Completed later in Phase 7 (2026-07-10/11) |

Remaining Phase 7 work: remove the three upstream-dependent read-back adapters recorded in [deletion-checklist.md](deletion-checklist.md).

## Blockers

### Upstream `Application::with_terminal` (optional)

Headless try-2 modals run through `HeadlessTvApp::exec_modal_view`. Interactive path uses `Application::new()` without try-1 snapshot populate.

### Headless `FileDialog::execute`

Upstream `FileDialog::execute(&mut Application)` is available for live `Application`, but the branch cannot currently construct a full upstream `Application` over the headless terminal because the required fields/constructors are private. The current headless `RunFileDialog` test path uses a Try-2-local queued adapter on `Try2Session`; it no longer consumes the try-1 `test_file_dialog_result` queue.

## Public docs rewrite (2026-07-10)

Public spec under `docs/pascal/std/tui/` updated for try-2 API:

- `README.md`, `session.md`, `terminal-checklist.md`
- `app/README.md`, `types.md`, `controls.md`, `modals.md`, `handlers.md`, `lifecycle.md`, `testing.md`, `message-box.md`, `vm-bridge.md`
- `docs/pascal/std/README.md` TUI example

Removed try-1 `Application.Create*` / `ExecDialog` / offset-band references from public docs. `CM_*` documented as canonical command ids; `Command.*` noted as legacy aliases.

## Phase 7 dead-code cleanup (2026-07-10)

- Deleted `tui/callbacks.rs` (try-1 command offset dispatch); `OnCommand` uses `try2/events.rs` only
- `command_map.rs` slimmed to reserved `CM_*` list + upstream sync test (no offset band)
- Deleted unused `try2/geometry.rs` (duplicate of `tv_geometry.rs`)
- Removed `dispatch_tui_command` from `lifecycle.rs`; dead `set_text_from_fpas` on checkbox/radio bridged views
- `message_box.rs` returns upstream command ids unchanged; headless `TestSetDialogResult` stub moved to `Try2Session`
- Dropped `Command.*` Pascal aliases; use `CM_*` constants (`command_api.rs` removed)
- Dropped `TurboVisionState` from `TuiState`; headless test hooks live on `Try2Session` only

## Phase 7 snapshot cleanup (2026-07-10)

- `TurboVisionObject` enum and widget snapshot structs removed from `shared/tui.rs`
- `TurboVisionState` removed; headless test hooks (`dialog_result`, `file_dialog_result`) live on `Try2Session`
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
