# Std.Tui VM bridge

This page tracks the public Pascal-to-VM bridge for contributors. Turbo Vision widget construction and the interactive run loop use the `Create*` facade, `session_app.rs`, and `tv_run.rs`.

**Upstream:** the VM bridge calls into [`turbo-vision`](https://crates.io/crates/turbo-vision) 2.0 from [turbo-vision-4-rust](https://github.com/aovestdipaperino/turbo-vision-4-rust) (git tag `v2.0.0` until crates.io publishes 2.x). Keep reserved `CM_*` command ids aligned with the pinned upstream version in `Cargo.lock` (see [Turbo Vision bump checklist](#turbo-vision-bump-checklist) below).

## Architecture

Expose a **Pascal-native** `Std.Tui` API (`Application.Create*`, `OnCommand`, `Run`) over upstream Turbo Vision. FPAS programs do not see Rust `View` traits, builders, or ownership.

```text
Pascal Application.*
    → VM intrinsics (crates/fpas-vm/src/vm/execute/io/tui/mod.rs)
    → TurboVisionState (crates/fpas-vm/src/vm/shared/tui.rs)
         HashMap<u32, TurboVisionObject>  — authoritative handle graph
    → projection at Run / reconcile
    → turbo_vision::app::Application  — one live instance per Open…Close on main worker
         (Worker.live_turbo_vision_app in session_app.rs)
```

| Concern | Upstream | FPAS bridge entry |
| --- | --- | --- |
| Widget types | `views::*` | `tv_views.rs`, `menu_build.rs` from FPAS snapshots |
| Event loop | `get_event`, `handle_event` | `session_app.rs` |
| Menu / status | `MenuBar`, `StatusLine` | `navigation.rs`, `chrome_layout.rs` |
| Modal execute | `Dialog::execute` | `exec_dialog.rs` (live session) |
| Message box | `helpers::msgbox::message_box` | `msgbox.rs` — [message-box.md](message-box.md) |
| File picker | `FileDialog` | `file_dialog.rs` (live session) |
| Command ids | Borland `CM_*` | `command_map.rs` + `fpas-std` `command_ids.rs` |

| Concern | Location | Notes |
| --- | --- | --- |
| Retained handles | `shared/tui.rs`, `control_create.rs` | `Create*` writes FPAS records; upstream widgets rebuilt at reconcile |
| Headless paint | `headless_tv_draw.rs` | Upstream TV `draw` → CRT export |
| Headless input | `HeadlessTvEventInbox`, `test_mouse.rs` | `TestClickMouse` via TV `handle_event` |
| Headless commands | `commands.rs`, `tv_run.rs` | Queue + `Pump` instead of TV event loop |
| Full desktop rebuild | `reconcile.rs`, `tv_run.rs` | `pending_reconcile` → wipe → repopulate; data mutations use `live_patch.rs` when possible |
| Desktop z-order | `tv_run.rs` | Windows and dialogs merged, sorted by handle, so newer windows stack above older dialogs |
| Live view maps | `live_patch.rs`, `tv_run.rs` | `live_view_ids` + `live_child_root_view_ids` (parent root `ViewId`) |
| State cells | `turbo_vision_*_cell.rs`, `bridged_*.rs` | Checkbox/radio/list/input sync; `Bridged*` for live `SetText` |
| Command offset band | `command_map.rs` | Colliding app ids use `0x8000` band; `Command.*` pass through |

## Turbo Vision bump checklist

On every `turbo-vision` tag or revision bump in `Cargo.lock`:

1. Run `cargo test -p fpas-vm reserved_list_matches_upstream` — update `TURBO_VISION_RESERVED_COMMANDS` in `command_map.rs` if it fails.
2. Confirm `fpas-std` `COMMAND_*` constants still match Borland `CM_*` for `Command.Quit`, `Close`, `Accept`, `Cancel`.
3. Run `fpas test tests/tui/controls/tui_turbo_vision_reserved_command_test.fpas` and `fpas test apps/ide/tests/`.

After any bridge change, also run [terminal checklist](../terminal-checklist.md) (`cargo test --workspace`, `tests/tui/controls/`, `apps/ide/tests/`).

Current public lowering includes:

| Pascal symbol | VM intrinsic |
| --- | --- |
| `Application.Open` | `TuiApplicationOpen` |
| `Application.Close` | `TuiApplicationClose` |
| `Application.Size` | `TuiApplicationSize` |
| `Application.Run` | `TuiApplicationRun` |
| `Application.Quit` | `TuiQuit` |
| `Application.CreateDialog` | `TuiCreateDialog` |
| `Application.CreateWindow` | `TuiCreateWindow` |
| `Application.CreateButton` | `TuiCreateButton` |
| `Application.CreateStaticText` | `TuiCreateStaticText` |
| `Application.CreateMemo` | `TuiCreateMemo` |
| `Application.CreateTextViewer` | `TuiCreateTextViewer` |
| `Application.CreateInputLine` | `TuiCreateInputLine` |
| `Application.CreateListBox` | `TuiCreateListBox` |
| `Application.CreateCheckBox` | `TuiCreateCheckBox` |
| `Application.CreateRadioButton` | `TuiCreateRadioButton` |
| `Application.AddChild` | `TuiAddChild` |
| `Application.AddWindow` | `TuiAddWindow` |
| `Application.CreateMenuBar` | `TuiCreateMenuBar` |
| `Application.SetMenuBar` | `TuiSetMenuBar` |
| `Application.SetMenus` | `TuiSetMenus` |
| `Application.CreateStatusLine` | `TuiCreateStatusLine` |
| `Application.SetStatusLine` | `TuiSetStatusLine` |
| `Application.SetStatusItems` | `TuiSetStatusItems` |
| `Application.SetText` | `TuiSetText` |
| `Application.SetChecked` | `TuiSetChecked` |
| `Application.SetItems` | `TuiSetItems` |
| `Application.SetTitle` | `TuiSetTitle` |
| `Application.RunFileDialog` | `TuiRunFileDialog` |
| `Application.TestSetFileDialogResult` | `TuiTestSetFileDialogResult` |
| `Application.ExecDialog` | `TuiExecDialog` |
| `Application.MessageBox` | `TuiMessageBox` — see [message-box.md](message-box.md) |
| `Application.InputText` | `TuiInputText` |
| `Application.Checked` | `TuiChecked` |
| `Application.Selected` | `TuiSelected` |
| `Application.ListSelection` | `TuiListSelection` |
| `Application.TestSetDialogResult` | `TuiTestSetDialogResult` |
| `Application.OnCommand` | `TuiRegisterOnCommand` |
| `Application.OnKey` | `TuiRegisterOnKey` |
| `Application.OnMouse` | `TuiRegisterOnMouse` |
| `Application.Pump` | `TuiPump` |
| `Application.TestClickButton` | `TuiTestClickButton` |
| `Application.TestClickMouse` | `TuiTestClickMouse` |
| `Application.TestDispatchMenuCommand` | `TuiTestDispatchMenuCommand` |
| `Application.OpenForTest` | `TuiOpenForTest` |
| `Application.CloseForTest` | `TuiCloseForTest` |

Screen assertions in headless tests use [`Std.Test`](../../testing/test.md) `AssertScreenLine` and `AssertScreenCell` on the shared console back buffer.

## Rust module layout

Turbo Vision bridge code lives under `crates/fpas-vm/src/vm/execute/io/tui/`:

| Module | Responsibility |
| --- | --- |
| `lifecycle.rs` | `pop_tui_application`, session reset/close, `OnCommand` dispatch |
| `application.rs` | `Application.Open`, `Run`, `Size`, `Close` |
| `handles.rs` | Turbo Vision handle records and `Rect` decoding |
| `dialogs.rs` | `CreateDialog` |
| `windows.rs` | `CreateWindow`, `AddWindow` |
| `controls.rs` | `CreateButton`, `CreateStaticText`, `CreateMemo`, `CreateTextViewer`, `CreateInputLine`, `CreateListBox`, `CreateCheckBox`, `CreateRadioButton`, `AddChild`, runtime setters |
| `navigation.rs` | `CreateMenuBar`, `SetMenuBar`, `CreateStatusLine`, `SetStatusLine` |
| `menu_build.rs` | Upstream menu construction from FPAS `Menu` / `MenuItem` records |
| `callbacks.rs` | Turbo Vision command event to FPAS `OnCommand` |
| `tv_input_events.rs` | Unhandled Turbo Vision keyboard/mouse to FPAS `OnKey` / `OnMouse` |
| `session_app.rs` | Live turbo-vision `Application` on the main worker; shared by `Run`, `ExecDialog`, `RunFileDialog`; re-entrant interactive loop |
| `interactive_loop.rs` | Scripted interactive loop for Rust VM tests (`TurboVisionInteractiveSession` trait) |
| `commands.rs` | `Pump`, `Quit`, `TestClickButton`, `TestClickMouse`, `TestDispatchMenuCommand`, command queue |
| `test_mouse.rs` | Headless `TestClickMouse` hit testing for check boxes and radio buttons |
| `file_dialog.rs` | `RunFileDialog`, `TestSetFileDialogResult` |
| `exec_dialog.rs` | `ExecDialog`, `InputText`, `Checked`, `Selected`, `ListSelection`, `TestSetDialogResult` |
| `msgbox.rs` | `MessageBox` — upstream `helpers::msgbox::message_box` on live session |
| `bridged_button.rs` | `Button` view with live `SetText` patching |
| `bridged_check_box.rs` | Modal `CheckBox` view syncing checked state to FPAS |
| `bridged_list_box.rs` | Modal `ListBox` view syncing selected index to FPAS |
| `bridged_radio_button.rs` | Modal `RadioButton` view syncing selected state and FPAS group cells |
| `bridged_static_text.rs` | `StaticText` view with live `SetText` patching |
| `bridged_memo.rs` | `Memo` view with live `SetText` patching |
| `bridged_text_viewer.rs` | `TextViewer` view with live `SetText` patching |
| `tv_run.rs` | Terminal and headless `Application.Run` for Turbo Vision |
| `reconcile.rs` | Live widget-tree reconcile and headless CRT repaint |
| `live_patch.rs` | Incremental live updates for data mutations |
| `headless_tv_draw.rs` | Headless upstream TV `draw` → CRT export |
| `tv_headless_backend.rs` | In-memory `Backend` for headless TV `Terminal` |
| `testing.rs` | `OpenForTest`, `CloseForTest`, dialog test result seeding |
| `tui_run.rs` | `Application.Run` entry (Turbo Vision only) |

## See Also

- [Application](README.md)
- [Session](../session.md)
- [Terminal checklist](../terminal-checklist.md)
- [Message box](message-box.md)
