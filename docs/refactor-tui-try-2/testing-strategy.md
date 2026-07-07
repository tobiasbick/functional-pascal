# Testing strategy

Test plan for try-2. Goal: prove **user-visible behavior** and **thin bridge invariants**, not snapshot/reconcile internals.

## Current (landed on branch)

| Test | Path |
| --- | --- |
| Modal OK via `ExecView` | `tests/tui/smoke/modal_button_try2_test.fpas` |
| Run / `OnCommand` / `CM_QUIT` | `tests/tui/smoke/run_quit_try2_test.fpas` |
| Rust unit tests | `cargo test -p fpas-vm try2::` (registry, geometry, events, dialog, button, modals) |

try-1 `tests/tui/controls/*` (37 files) still run unchanged — coexistence routing in `try2_should_handle_application_run()`.

## Principles

1. **Prefer real upstream paths** — headless `exec_view` + `put_event` over `TestSetDialogResult`.
2. **One widget concern per test file** — `tests/tui/views/button_test.fpas`, not mega-files.
3. **Screen assertions via `Std.Test`** — unchanged: `AssertScreenLine`, `AssertScreenCell` after draw.
4. **Rust tests for registry and diagnostics** — invalid handles, double-close, callback re-entry.
5. **Delete, don’t port** tests that only validate try-1 bridge mechanics.

## Test directory layout (target)

Final names may drop the `_try2` suffix when try-1 tests are removed (phase 7).

```text
tests/tui/
  smoke/
    modal_button_try2_test.fpas   { landed — target: modal_button_test.fpas }
    run_quit_try2_test.fpas       { landed — target: run_quit_test.fpas }
  views/
    button_test.fpas
    dialog_test.fpas
    window_test.fpas
    input_line_test.fpas
    listbox_test.fpas
    checkbox_test.fpas
    radio_button_test.fpas
    static_text_test.fpas
    memo_test.fpas
    text_viewer_test.fpas
  chrome/
    menu_bar_test.fpas
    status_line_test.fpas
  modals/
    message_box_test.fpas
    file_dialog_test.fpas
  events/
    on_key_test.fpas
    on_mouse_test.fpas
    command_ids_test.fpas    { CM_OK, CM_QUIT pass through unchanged }
```

Update [`tests/suite.fpasprj`](../tests/suite.fpasprj) when old paths are removed.

## Tests to delete (try-1)

Remove when try-2 replacement exists:

| Old file | Reason |
| --- | --- |
| `tui_turbo_vision_live_tree_test.fpas` | Tests reconcile/live tree |
| `tui_turbo_vision_live_dialog_test.fpas` | Same |
| `tui_turbo_vision_reserved_command_test.fpas` | Offset band removed |
| `tui_turbo_vision_spike_test.fpas` | Spike / internal |
| `tui_turbo_vision_chrome_paint_test.fpas` | Repaint flag internals — replace with screen assert |
| All `set_text_*` if covered by `views/*_test.fpas` | Consolidate |

Keep behavioral coverage, not file names.

## Headless pattern (target)

```pascal
program ButtonModalTest;

uses Std.Tui, Std.Test;

procedure FailOnCommand(App: Application; Cmd: CommandId);
begin
  AssertEquals(CM_OK, Cmd)
end;

begin
  var App := Application.OpenForTest(40, 14);
  var Dlg := Dialog.NewModal(Bounds(5, 3, 30, 8), 'Test');
  var Btn := Button.New(Bounds(10, 4, 10, 2), 'OK', CM_OK, true);
  Dlg.Add(Btn);
  Test.Click(App, Btn);   { interim: Application.TestClickButton(App, Btn) }
  var Cmd := Application.ExecView(App, Dlg);
  AssertEquals(CM_OK, Cmd);
  Application.CloseForTest(App)
end.
```

### `Test.*` helpers (FPAS)

| Helper | Behavior |
| --- | --- |
| `Test.Click(App, Button)` | Synthesize mouse down/up at button center; `handle_event` |
| `Test.InjectEvent(App, …)` | Low-level; map from `Std.Console` event records |
| `Test.Pump(App)` | **Optional** — one `get_event` iteration without blocking; use sparingly |

**Remove:** `TestSetDialogResult`, `TestSetFileDialogResult`, `TestClickButton` (rename to `Test.Click`), `TestDispatchMenuCommand` → `Test.DispatchMenu` if still needed.

## Rust tests (`crates/fpas-vm`)

| Module | Cases |
| --- | --- |
| `registry.rs` | invalid handle, use-after-close, duplicate free |
| `geometry.rs` | rect round-trip |
| `events.rs` | command ids pass through unchanged ✅; full callback dispatch covered by FPAS smoke |
| `headless.rs` | draw produces non-empty buffer (via `HeadlessTvApp` + modal smoke) |

Keep tests in the same file as the module when under ~100 lines; otherwise `#[cfg(test)] mod tests` in submodule.

## Regression command list

After each phase:

```bash
cargo fmt
cargo build
cargo test --workspace
fpas fmt --check tests/tui/ apps/ide/
fpas test tests/tui/
fpas test apps/ide/tests/
```

Final: `fpas test tests/` or `cargo test -p fpas-cli fpas_regression_suite_passes`.

## Interactive manual checklist

From [terminal-checklist.md](../pascal/std/tui/terminal-checklist.md) — run after phase 2 (try-2 vertical slice):

```bash
fpas run examples/pascal/tui/modal_button_try2.fpas
```

- [ ] Modal: OK and Cancel with mouse click
- [ ] Modal: Enter on default OK, Esc on Cancel
- [ ] Mouse click on check box and radio button
- [ ] Menu accelerator and pull-down command
- [ ] Window behind dialog z-order
- [ ] IDE About message box
- [ ] IDE Open file dialog
- [ ] Terminal resize during `Run` (if upstream handles)

## Coverage gaps acceptable in v1

- `EditorWindow`, help system, color dialog
- SSH / remote keys (`TV_REMOTE_KEYS`)
- Palette customization
- Tile/cascade window management

Document gaps in `docs/pascal/std/tui/README.md` — not as unimplemented promises in method docs.

## Golden files

Try-1 goldens tied to old screen layout may break. Prefer:

- `AssertScreenLine` substring checks over full golden CRT dumps, or
- Regenerate goldens once after API stabilizes.
