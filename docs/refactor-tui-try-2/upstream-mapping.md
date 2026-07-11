# Upstream mapping

Reference for binding `turbo-vision` 2.0 (`v2.0.0`) to the try-2 FPAS API. Refresh this table on every upstream tag bump.

**Sources:** [turbo-vision-4-rust](https://github.com/aovestdipaperino/turbo-vision-4-rust) tag `v2.0.0` — `src/lib.rs`, `src/views/`, `src/app/application.rs`, `src/helpers/`.

## Implementation status (branch `refactor/tui-try-2`, 2026-07-11)

| FPAS symbol | Status | VM location |
| --- | --- | --- |
| `Dialog.NewModal` | ✅ | `try2/views/dialog.rs` |
| `Button.New`, `Dialog.Add` | ✅ | `try2/views/button.rs` |
| `Application.ExecView` | ✅ | `try2/modals.rs`, `try2/headless.rs` |
| `Application.Run` + `OnCommand` | ✅ | `try2/run.rs` — `Application.Run(App)` or `Application.Run(App, OnCommand)` (intrinsic 484) |
| `Application.Quit` | ✅ | `try2/commands.rs`, `try2/run.rs` |
| `Test.Click` | ✅ | `try2/testing.rs` |
| `Test.DispatchMenu` | ✅ | `try2/testing.rs` |
| `Test.InjectCommand` | ✅ | `try2/intrinsics.rs` |
| `Test.InjectKeyboard` | ✅ | `try2/intrinsics.rs` |
| `CM_OK`, `CM_CANCEL`, `CM_CLOSE`, `CM_QUIT`, `CM_OPEN`, `CM_ABOUT`, `CM_USER` | ✅ | sema + `fpas-std/tui/cm_constants.rs`; compiler built-in constants |
| `Window.New`, `Window.Add`, `Desktop.Add` | ✅ | `try2/views/window.rs`, `try2/views/desktop.rs` |
| `StaticText.New`, `Dialog.Add` / `Window.Add` | ✅ | `try2/views/static_text.rs`, `try2/views/attach.rs` |
| `MenuBar.New`, `StatusLine.New`, `SetMenuBar`, `SetStatusLine` | ✅ | `try2/chrome.rs` |
| `InputLine`, `ListBox`, `CheckBox`, `RadioButton`, `Memo`, `TextViewer` | ✅ | `try2/views/*` |
| `Application.MessageBox` | ✅ | `try2/message_box.rs` |
| `Application.RunFileDialog` | ✅ with local headless adapter | `try2/file_dialog.rs` — live route uses upstream `FileDialog::execute`; headless consumes Try-2 session state because upstream `FileDialog::execute` needs a full `Application` |
| `Application.OnKey`, `Application.OnMouse` | ✅ | `try2/events.rs`, `try2/input_events.rs` |

## Application and session

| Upstream Rust | FPAS try-2 | VM module (landed → target) |
| --- | --- | --- |
| `Application::new()` | `Application.New` / first interactive op | `try2/application_intrinsics.rs`, `try2/lifecycle.rs`, `try2/session_app.rs` |
| `Application::run()` | `Application.Run` | `try2/run.rs` |
| `Application::get_event()` | Internal run loop | `try2/run.rs` |
| `Application::handle_event()` | Internal + callbacks | `try2/events.rs`, `try2/input_events.rs` |
| `Application::exec_view(Box<dyn View>)` | `Application.ExecView` | `try2/modals.rs`, `try2/headless.rs` |
| `Application::set_menu_bar` | `Application.SetMenuBar` | `try2/chrome.rs` |
| `Application::set_status_line` | `Application.SetStatusLine` | `try2/chrome.rs` |
| `Application::put_event` | `Test.InjectCommand` / `Test.InjectKeyboard` (headless) | `try2/intrinsics.rs`, `try2/testing.rs` |
| `app.running = false` | `Application.Quit` | `try2/commands.rs`, `try2/run.rs` |
| `terminal.size()` | `Application.Size` | `try2/application_intrinsics.rs`, `try2/application_records.rs` |
| `helpers::msgbox::message_box` | `Application.MessageBox` | `try2/message_box.rs` |

## Desktop and groups

| Upstream Rust | FPAS try-2 | Notes |
| --- | --- | --- |
| `desktop::Desktop::add` | `Desktop.Add(App, Window)` | Window must be `Box<Window>` upstream |
| `group::Group::add` | `Dialog.Add` / `Window.Add` | Parent must be group-like |
| `Dialog::new` | `Dialog.New` | Modeless dialog shell |
| `Dialog::new_modal` | `Dialog.NewModal` | Sets `SF_MODAL` |
| `Window::new` | `Window.New` | |
| `Dialog::execute` | Used inside `exec_view` | Do not expose separately initially |

## Controls and chrome (landed through Phase 4)

| Upstream type | FPAS | Status |
| --- | --- | --- |
| `button::Button` | `Button` | ✅ `Button.New`, `Dialog.Add` |
| `dialog::Dialog` | `Dialog` | ✅ `Dialog.NewModal` only |
| `static_text::StaticText` | `StaticText` | ✅ `try2/views/static_text.rs` |
| `input_line::InputLine` | `InputLine` | ✅ `try2/views/input_line.rs` |
| `listbox::ListBox` | `ListBox` | ✅ `try2/views/list_box.rs` |
| `checkbox::CheckBox` | `CheckBox` | ✅ `try2/views/check_box.rs` |
| `radiobutton::RadioButton` | `RadioButton` | ✅ `try2/views/radio_button.rs` |
| `memo::Memo` | `Memo` | ✅ `try2/views/memo.rs` |
| `text_viewer::TextViewer` | `TextViewer` | ✅ `try2/views/text_viewer.rs` |
| `window::Window` | `Window` | ✅ `Window.New`, `Window.Add`, `Desktop.Add` |
| `menu_bar::MenuBar` | `MenuBar` | ✅ `try2/chrome.rs` |
| `status_line::StatusLine` | `StatusLine` | ✅ `try2/chrome.rs` |

## Upstream controls not exposed by the FPAS facade

| Upstream type | FPAS | Priority |
| --- | --- | --- |
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

**Try-2 callbacks:** `OnCommand` receives the same integer upstream emitted. There is no command-map translation layer.

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

VM converts FPAS width/height to upstream corner rect in [`try2/geometry.rs`](../../crates/fpas-vm/src/vm/execute/io/tui/try2/geometry.rs).

## View trait (not exposed)

These stay in Rust only:

- `View::draw`, `handle_event`, `bounds`, `set_bounds`
- `View::as_any` / `as_any_mut` — not a supported read-back route for the three bridged controls at the pinned upstream revision
- `IdleView::idle` — overlay widgets if needed later

## test-util feature

Workspace already enables `features = ["test-util"]` on `turbo-vision`. During implementation, inventory:

- `test_util` module path at tag `v2.0.0`
- Helpers for synthetic events, in-memory terminal, assertions

Map each to FPAS `Test.*` helpers or delete FPAS stubs if redundant.

## Bump checklist (per upstream release)

1. Diff `core/command.rs` — add/remove `CM_*` in `fpas-std` and sema constants.
2. Diff `views/` — new widgets → new row in the controls table.
3. Run `cargo test -p fpas-vm` bridge tests.
4. Run `fpas test tests/tui/`.
5. Run `fpas test apps/ide/tests/`.
6. Update [`docs/pascal/std/tui/`](../pascal/std/tui/) — not this plan — when bump lands.

## Intrinsic naming convention (try-2)

Land in `fpas-bytecode/src/intrinsic/tui/variants/try2.inc` (473+). Current:

```text
DialogNewModal = 473
ExecView = 475
TestInjectKeyboard = 476
ButtonNew = 477
DialogAdd = 478
TestInjectCommand = 479
WindowNew = 480
WindowAdd = 481
DesktopAdd = 482
StaticTextNew = 483
ApplicationRunWithOnCommand = 484
MenuBarNew = 485
StatusLineNew = 486
CheckBoxNew = 487
InputLineNew = 488
CheckBoxChecked = 489
CheckBoxSetChecked = 490
InputLineText = 491
InputLineSetText = 492
ListBoxNew = 493
ListBoxSelection = 494
ListBoxSetItems = 495
RadioButtonNew = 496
RadioButtonSelected = 497
RadioButtonSetSelected = 498
MemoNew = 499
MemoSetText = 500
TextViewerNew = 501
TextViewerSetText = 502
```

Future widgets: continue the try-2 range, group by widget, and keep related read-back/setter intrinsics adjacent.
