# Terminal UI

Terminal UI APIs for Functional Pascal.

## Turbo Vision facade

`Std.Tui` is the Turbo Vision application facade: dialogs, windows, menus, controls, file pickers, and IDE-style chrome. Views are opaque handles created with `Dialog.NewModal`, `Button.New`, `Window.New`, and related `*.New` factories. User actions arrive as integer command ids through `Application.Run(App, OnCommand)`. Interactive modals (`ExecView`, `MessageBox`, `RunFileDialog`) run on the same upstream session as `Run`.

**Upstream:** [`Std.Tui`](app/README.md) is implemented over [`turbo-vision`](https://crates.io/crates/turbo-vision) 2.0 from [turbo-vision-4-rust](https://github.com/aovestdipaperino/turbo-vision-4-rust) (workspace git dependency on tag `v2.0.0`). Three checkbox/radio/outline bridge adapters remain in the VM until upstream exposes live read-back — see [VM bridge](app/vm-bridge.md) and [tui-bridged-readback.md](../../../future/tui-bridged-readback.md). Agents should periodically check upstream for a fix ([AGENTS.md](../../../../AGENTS.md#upstream-watch--turbo-vision-4-rust-read-back-stream-a)); contributors may close this gap via [AI_CONTRIBUTING.md](../../../../AI_CONTRIBUTING.md#good-entry-points).

Simple terminal programs that draw every cell themselves (fullscreen explorers, animations, custom loops) use [`Std.Console`](../console/README.md) with raw mode, alternate screen, and `ReadEventTimeout` / `PollEvent`.

### Examples

| Style | Starting points |
| --- | --- |
| Turbo Vision | `examples/pascal/tui/turbo_vision_dialog.fpas`, `examples/pascal/tui/turbo_vision_window.fpas`, `apps/ide` |
| Console event loop | `examples/math/mandelbrot/mandelbrot.fpasprj`, `examples/math/julia/julia.fpasprj` |

| Topic | Description |
| --- | --- |
| [Session API](session.md) | `Application.Open`, `Run`, `Close`, size |
| [Application](app/README.md) | `Application.*` lifecycle and helpers |
| [Types](app/types.md) | Handles, `TuiRect`, `CM_*` constants, menu/status records |
| [Controls](app/controls.md) | Buttons, text fields, lists, check boxes, chrome |
| [Dialogs and windows](app/modals.md) | `Dialog`, `Window`, `ExecView`, attachment |
| [File dialog](app/file-dialog.md) | Modal `Application.RunFileDialog` |
| [Handlers](app/handlers.md) | `Run(App, OnCommand)`, `OnKey`, `OnMouse` |
| [Lifecycle](app/lifecycle.md) | Open, run, quit, close, shared live session |
| [Native testing](app/testing.md) | Headless tests with `OpenForTest` |
| [VM bridge](app/vm-bridge.md) | Pascal-to-intrinsic map (contributors) |
| [Terminal checklist](terminal-checklist.md) | Local verification commands |
| [Cell width](cell-width.md) | Unicode display-width policy |

## See Also

- [`Std.Console`](../console/README.md)
- [Std index](../README.md)
