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
| `Application.OnCommand` | `TuiOnCommand` |
| `Application.Pump` | `TuiPump` |
| `Application.TestClickButton` | `TuiTestClickButton` |
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
| `controls.rs` | `CreateButton`, `CreateStaticText`, `CreateInputLine`, `CreateListBox`, `CreateCheckBox`, `CreateRadioButton`, `AddChild` |
| `navigation.rs` | `CreateMenuBar`, `SetMenuBar`, `CreateStatusLine`, `SetStatusLine` |
| `callbacks.rs` | Turbo Vision command event to FPAS `OnCommand` |
| `commands.rs` | `Pump`, `Quit`, `TestClickButton`, command queue |
| `tv_run.rs` | Terminal and headless `Application.Run` for Turbo Vision |
| `events.rs` | Headless `TestSend*` event injection |
| `testing.rs` | `OpenForTest`, `TestPump*`, `CloseForTest` |
| `host/` | Hosted global-handler loop (`HostRegister*`, `HostProcessNext`) |

## See Also

- [Application](README.md)
- [Future Turbo Vision plan](../../../../future/turbo-vision-4-rust/README.md)
