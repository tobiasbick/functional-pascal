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
| `Application.CreateButton` | `TuiCreateButton` |
| `Application.AddChild` | `TuiAddChild` |
| `Application.OnCommand` | `TuiOnCommand` |
| `Application.Pump` | `TuiPump` |
| `Application.TestClickButton` | `TuiTestClickButton` |
| `Application.QueryScreenSize` | `TuiQueryScreenSize` |
| `Application.QueryScreenLine` | `TuiQueryScreenLine` |
| `Application.QueryScreenCell` | `TuiQueryScreenCell` |

## See Also

- [Application](README.md)
- [Future Turbo Vision plan](../../../../future/turbo-vision-4-rust/README.md)
