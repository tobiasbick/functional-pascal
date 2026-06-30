# Std.Tui VM bridge

This page tracks the public Pascal-to-VM bridge for contributors.

The old public `Application.Host*` view, modal, and widget bridge has been removed from bytecode and the VM `views/*` module. The transitional retained `ViewRegistry` remains only for the host event loop until a follow-up removes it from `fpas-std` and `TuiState`.

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
