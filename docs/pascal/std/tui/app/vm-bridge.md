# Std.Tui VM bridge

This page tracks the public Pascal-to-VM bridge for contributors.

The old public `Application.Host*` view, modal, and widget bridge has been removed from bytecode, the VM, and `fpas-std`. The hosted loop supports global handlers (`HostRegisterOnPaint`, `HostRegisterOnKeyPressed`, `HostBindCommand`, and related calls) plus screen queries. Turbo Vision widget construction uses the separate `Create*` facade.

Current public lowering includes:

| Pascal symbol | VM intrinsic |
| --- | --- |
| `Application.Open` | `TuiApplicationOpen` |
| `Application.Close` | `TuiApplicationClose` |
| `Application.Configure` | `TuiApplicationConfigure` |
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
| `Application.CreateStatusLine` | `TuiCreateStatusLine` |
| `Application.SetStatusLine` | `TuiSetStatusLine` |
| `Application.RunFileDialog` | `TuiRunFileDialog` |
| `Application.TestSetFileDialogResult` | `TuiTestSetFileDialogResult` |
| `Application.ExecDialog` | `TuiExecDialog` |
| `Application.InputText` | `TuiInputText` |
| `Application.Checked` | `TuiChecked` |
| `Application.Selected` | `TuiSelected` |
| `Application.TestSetDialogResult` | `TuiTestSetDialogResult` |
| `Application.OnCommand` | `TuiOnCommand` |
| `Application.OnKey` | `TuiRegisterOnKey` |
| `Application.OnMouse` | `TuiRegisterOnMouse` |
| `Application.Pump` | `TuiPump` |
| `Application.TestClickButton` | `TuiTestClickButton` |
| `Application.TestDispatchMenuCommand` | `TuiTestDispatchMenuCommand` |
| `Application.QueryScreenSize` | `TuiQueryScreenSize` |
| `Application.QueryScreenLine` | `TuiQueryScreenLine` |
| `Application.QueryScreenCell` | `TuiQueryScreenCell` |

## Rust module layout

Turbo Vision bridge code lives under `crates/fpas-vm/src/vm/execute/io/tui/`:

| Module | Responsibility |
| --- | --- |
| `application.rs` | `Application.Open`, `Configure`, `Run`, `Size`, `RequestRedraw` |
| `handles.rs` | Turbo Vision handle records and `Rect` decoding |
| `dialogs.rs` | `CreateDialog` |
| `windows.rs` | `CreateWindow`, `AddWindow` |
| `controls.rs` | `CreateButton`, `CreateStaticText`, `CreateMemo`, `CreateTextViewer`, `CreateInputLine`, `CreateListBox`, `CreateCheckBox`, `CreateRadioButton`, `AddChild` |
| `navigation.rs` | `CreateMenuBar`, `SetMenuBar`, `CreateStatusLine`, `SetStatusLine` |
| `menu_build.rs` | Upstream menu construction from FPAS `Menu` / `MenuItem` records |
| `callbacks.rs` | Turbo Vision command event to FPAS `OnCommand` |
| `tv_input_events.rs` | Unhandled Turbo Vision keyboard/mouse to FPAS `OnKey` / `OnMouse` |
| `interactive_loop.rs` | Pluggable interactive run loop; dispatches commands and opt-in input hooks |
| `commands.rs` | `Pump`, `Quit`, `TestClickButton`, `TestDispatchMenuCommand`, command queue |
| `file_dialog.rs` | `RunFileDialog`, `TestSetFileDialogResult` |
| `exec_dialog.rs` | `ExecDialog`, `InputText`, `Checked`, `Selected`, `TestSetDialogResult` |
| `bridged_check_box.rs` | Modal `CheckBox` view syncing checked state to FPAS |
| `bridged_radio_button.rs` | Modal `RadioButton` view syncing selected state to FPAS |
| `tv_run.rs` | Terminal and headless `Application.Run` for Turbo Vision. The terminal loop steps the Turbo Vision event pump itself so commands left unhandled by Turbo Vision (buttons, menus, status items) are routed into the FPAS `OnCommand` callback, and an `Application.Quit` from that callback ends the loop. Keyboard and mouse events still typed after `handle_event` can reach opt-in `Application.OnKey` / `Application.OnMouse` handlers. `turbo_vision_populate_desktop` builds the desktop roots and is shared with the live reconcile rebuild. |
| `reconcile.rs` | Live widget-tree reconcile during a run. FPAS mutations set a dirty flag; after each event step the whole desktop is rebuilt from current FPAS state so new roots and children added to already-shown roots both appear. Menu bar and status line are re-synced from FPAS state on the same dirty pass. Headless runs repaint the CRT buffer instead. |
| `events.rs` | Headless `TestSend*` event injection |
| `testing.rs` | `OpenForTest`, `TestPump*`, `CloseForTest` |
| `host/` | Hosted global-handler loop (`HostRegister*`, `HostProcessNext`) |

## See Also

- [Application](README.md)
- [Future Turbo Vision plan](../../../../future/turbo-vision-4-rust/README.md)
