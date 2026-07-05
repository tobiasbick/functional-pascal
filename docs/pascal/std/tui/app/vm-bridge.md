# Std.Tui VM bridge

This page tracks the public Pascal-to-VM bridge for contributors. Turbo Vision widget construction and the interactive run loop use the `Create*` facade, `session_app.rs`, and `tv_run.rs`.

**Upstream:** the VM bridge calls into [`turbo-vision`](https://crates.io/crates/turbo-vision) 2.0 from [turbo-vision-4-rust](https://github.com/aovestdipaperino/turbo-vision-4-rust) (git tag `v2.0.0` until crates.io publishes 2.x). Keep reserved `CM_*` command ids aligned with the pinned upstream version in `Cargo.lock`. Planned bridge work: [docs/refactor/tui-bridge/](../../../refactor/tui-bridge/00-context.md).

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
| `Application.MessageBox` | `TuiMessageBox` (registered; no public Pascal spec page until [07](../../../refactor/tui-bridge/07-pascal-message-box-api.md)) |
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
| `bridged_check_box.rs` | Modal `CheckBox` view syncing checked state to FPAS |
| `bridged_list_box.rs` | Modal `ListBox` view syncing selected index to FPAS |
| `bridged_radio_button.rs` | Modal `RadioButton` view syncing selected state to FPAS |
| `radio_button_mouse.rs` | Left-click select for live Turbo Vision radio buttons |
| `tv_run.rs` | Terminal and headless `Application.Run` for Turbo Vision |
| `reconcile.rs` | Live widget-tree reconcile and headless CRT repaint |
| `headless_tv_draw.rs` | Headless upstream TV `draw` → CRT export ([refactor 04](../../../refactor/tui-bridge/done/04-headless-test-util.md)) |
| `tv_headless_backend.rs` | In-memory `Backend` for headless TV `Terminal` |
| `testing.rs` | `OpenForTest`, `CloseForTest`, dialog test result seeding |
| `tui_run.rs` | `Application.Run` entry (Turbo Vision only) |

Contributor refactor backlog (bridge internals, not user spec): [docs/refactor/tui-bridge/](../../../refactor/tui-bridge/00-context.md).

## See Also

- [Application](README.md)
- [Terminal checklist](../terminal-checklist.md)
- [Refactor 03 — message_box (done)](../../../refactor/tui-bridge/done/03-about-message-box.md)
