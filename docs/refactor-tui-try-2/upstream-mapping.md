# Upstream mapping

Reference for binding `turbo-vision` 2.0 (`v2.0.0`) to the try-2 FPAS API. Refresh this table on every upstream tag bump.

**Sources:** [turbo-vision-4-rust](https://github.com/aovestdipaperino/turbo-vision-4-rust) tag `v2.0.0` — `src/lib.rs`, `src/views/`, `src/app/application.rs`, `src/helpers/`.

## Implementation status (branch `refactor/tui-try-2`, 2026-07-08)

| FPAS symbol | Status | VM location |
| --- | --- | --- |
| `Dialog.NewModal` | ✅ | `try2/views/dialog.rs` |
| `Button.New`, `Dialog.Add` | ✅ | `try2/views/button.rs` |
| `Dialog.AddButton` | ✅ convenience | `try2/views/button.rs` |
| `Application.ExecView` | ✅ | `try2/modals.rs`, `try2/headless.rs` |
| `Application.Run` + `OnCommand` | ✅ | `try2/run.rs` — `Application.Run(App)` or `Application.Run(App, OnCommand)` (intrinsic 484) |
| `Application.Quit` | ✅ via try-1 | `quit_requested`; try-2 run loop honors it |
| `Application.TestClickButton` | ✅ try-2 path | `try2/testing.rs` |
| `Try2InjectCommand` / `Try2InjectKeyboard` | ✅ interim | `try2/intrinsics.rs` — remove when `Test.InjectEvent` lands |
| `CM_OK`, `CM_CANCEL`, `CM_QUIT` | ✅ | sema + `fpas-std/tui/cm_constants.rs` |
| `Window.New`, `Window.Add`, `Desktop.Add` | ✅ | `try2/views/window.rs`, `try2/views/desktop.rs` |
| `StaticText.New`, `Dialog.Add` / `Window.Add` | ✅ | `try2/views/static_text.rs`, `try2/views/attach.rs` |
| `MenuBar.New`, `StatusLine.New`, `SetMenuBar`, `SetStatusLine` | ✅ | `try2/chrome.rs` |

## Application and session

| Upstream Rust | FPAS try-2 | VM module (landed → target) |
| --- | --- | --- |
| `Application::new()` | `Application.New` / first interactive op | `try2/app.rs` → `session.rs` |
| `Application::run()` | `Application.Run` | `try2/run.rs` → `run.rs` |
| `Application::get_event()` | Internal run loop | `try2/run.rs` |
| `Application::handle_event()` | Internal + callbacks | `try2/events.rs` → `events.rs` |
| `Application::exec_view(Box<dyn View>)` | `Application.ExecView` | `try2/modals.rs` → `modals.rs` |
| `Application::set_menu_bar` | `Application.SetMenuBar` | `chrome.rs` |
| `Application::set_status_line` | `Application.SetStatusLine` | `chrome.rs` |
| `Application::put_event` | `Test.InjectEvent` (headless) | `headless.rs` |
| `app.running = false` | `Application.Quit` | `run.rs` |
| `terminal.size()` | `Application.Size` | `session.rs` |
| `helpers::msgbox::message_box` | `Application.MessageBox` | `modals.rs` |

## Desktop and groups

| Upstream Rust | FPAS try-2 | Notes |
| --- | --- | --- |
| `desktop::Desktop::add` | `Desktop.Add(App, Window)` | Window must be `Box<Window>` upstream |
| `group::Group::add` | `Dialog.Add` / `Window.Add` | Parent must be group-like |
| `Dialog::new` | `Dialog.New` | Modeless dialog shell |
| `Dialog::new_modal` | `Dialog.NewModal` | Sets `SF_MODAL` |
| `Window::new` | `Window.New` | |
| `Dialog::execute` | Used inside `exec_view` | Do not expose separately initially |

## Controls (phase 2 vertical slice — partial)

| Upstream type | FPAS | Status |
| --- | --- | --- |
| `button::Button` | `Button` | ✅ `Button.New`, `Dialog.Add` |
| `dialog::Dialog` | `Dialog` | ✅ `Dialog.NewModal` only |
| `static_text::StaticText` | `StaticText` | ✅ `try2/views/static_text.rs` |
| `input_line::InputLine` | `InputLine` | ❌ phase 4 |
| `listbox::ListBox` | `ListBox` | ❌ phase 4 |
| `checkbox::CheckBox` | `CheckBox` | ❌ phase 4 |
| `radiobutton::RadioButton` | `RadioButton` | ❌ phase 4 |
| `memo::Memo` | `Memo` | ❌ phase 4 |
| `text_viewer::TextViewer` | `TextViewer` | ❌ phase 4 |
| `window::Window` | `Window` | ✅ `Window.New`, `Window.Add`, `Desktop.Add` |
| `menu_bar::MenuBar` | `MenuBar` | ✅ `try2/chrome.rs` |
| `status_line::StatusLine` | `StatusLine` | ✅ `try2/chrome.rs` |

## Controls (later phases)

| Upstream type | FPAS | Priority |
| --- | --- | --- |
| `outline::Outline` | `Outline` | Medium — IDE may not need immediately |
| `file_dialog::FileDialog` | `Application.RunFileDialog` | High |
| `editor::EditorWindow` | `EditorWindow` | Low |
| `help_window::HelpWindow` | — | Low |
| `color_dialog::ColorDialog` | — | Low |
| `scroller::Scroller` | — | Low |
| `cluster::Cluster` | Could replace manual `GroupId` radios | Evaluate |

## Commands

Export constants from `turbo_vision::core::command` (see upstream `prelude`).

| Category | Examples | FPAS |
| --- | --- | --- |
| App lifecycle | `CM_QUIT`, `CM_CLOSE` | `CM_QUIT`, `CM_CLOSE` |
| Dialog | `CM_OK`, `CM_CANCEL`, `CM_YES`, `CM_NO` | same |
| File | `CM_OPEN`, `CM_SAVE`, `CM_SAVE_AS`, … | same |
| Edit | `CM_UNDO`, `CM_CUT`, `CM_COPY`, `CM_PASTE` | same |
| View | `CM_ZOOM_IN`, `CM_TILE`, `CM_CASCADE` | same |
| Help | `CM_HELP_INDEX`, `CM_ABOUT` | same |

**Removed:** `command_map.rs` offset translation. FPAS `OnCommand` receives the same integer upstream emitted.

**Convention:** application-private commands start at `CM_USER` (4096) or another documented base.

## Events

| Upstream `Event` / `EventType` | FPAS |
| --- | --- |
| `EventType::Command` | `OnCommand` |
| `EventType::KeyDown` / key variants | `OnKey` → `Std.Console.KeyEvent` |
| Mouse events | `OnMouse` → `Std.Console.Event` |

## Geometry

| Upstream `Rect` | FPAS `Rect` |
| --- | --- |
| `Rect::new(x1, y1, x2, y2)` corners | `record x, y, width, height` |

VM converts FPAS width/height to upstream corner rect (reuse logic from [`tv_geometry.rs`](../../crates/fpas-vm/src/vm/execute/io/tui/tv_geometry.rs)).

## View trait (not exposed)

These stay in Rust only:

- `View::draw`, `handle_event`, `bounds`, `set_bounds`
- `View::as_any` / `as_any_mut` — used internally for `SetText` / read-back
- `IdleView::idle` — overlay widgets if needed later

## test-util feature

Workspace already enables `features = ["test-util"]` on `turbo-vision`. During implementation, inventory:

- `test_util` module path at tag `v2.0.0`
- Helpers for synthetic events, in-memory terminal, assertions

Map each to FPAS `Test.*` helpers or delete FPAS stubs if redundant.

## Bump checklist (per upstream release)

1. Diff `core/command.rs` — add/remove `CM_*` in `fpas-std` and sema constants.
2. Diff `views/` — new widgets → new row in phase 2 table.
3. Run `cargo test -p fpas-vm` bridge tests.
4. Run `fpas test tests/tui/`.
5. Run `fpas test apps/ide/tests/`.
6. Update [`docs/pascal/std/tui/`](../pascal/std/tui/) — not this plan — when bump lands.

## Intrinsic naming convention (try-2)

Land in `fpas-bytecode/src/intrinsic/tui/variants/try2.inc` (473+). Current:

```text
DialogNewModal = 473
DialogAddButton = 474
ExecView = 475
Try2InjectKeyboard = 476
ButtonNew = 477
DialogAdd = 478
Try2InjectCommand = 479
```

Future widgets: `TuiWindowNew`, `TuiDesktopAdd`, `TuiButtonSetText`, … — group by widget; keep alphabetical within the try-2 section.
