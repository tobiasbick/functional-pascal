# Std.Tui VM bridge

This page tracks the public Pascal-to-VM bridge for contributors.

The old public `Application.Host*` bridge has been de-registered from Sema and is no longer lowered by the compiler. Some internal VM code still exists until the runtime cleanup phase removes the retained implementation.

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
