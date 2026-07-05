# Terminal UI

Terminal UI APIs for Functional Pascal.

## Turbo Vision facade

`Std.Tui` is the Turbo Vision application facade: dialogs, windows, menus, buttons, file pickers, and IDE-style chrome. Widgets are retained handles (`Application.CreateDialog`, `CreateButton`, `AddChild`, …). User actions arrive as integer commands through `Application.OnCommand`, and the runtime drives the Turbo Vision event pump via `Application.Run`. Interactive modals (`ExecDialog`, `RunFileDialog`) run on the same upstream session as `Run`.

**Upstream:** [`Std.Tui`](app/README.md) is implemented over [`turbo-vision`](https://crates.io/crates/turbo-vision) 2.0 from [turbo-vision-4-rust](https://github.com/aovestdipaperino/turbo-vision-4-rust) (workspace git dependency on tag `v2.0.0`).

Simple terminal programs that draw every cell themselves (fullscreen explorers, animations, custom loops) use [`Std.Console`](../console/README.md) with raw mode, alternate screen, and `ReadEventTimeout` / `PollEvent`.

### Examples

| Style | Starting points |
| --- | --- |
| Turbo Vision | `examples/pascal/tui/turbo_vision_dialog.fpas`, `apps/ide` |
| Console event loop | `examples/math/mandelbrot/mandelbrot.fpasprj`, `examples/math/julia/julia.fpas` |

| Topic | Description |
| --- | --- |
| [Session API](session.md) | `Application.Open`, `Run`, `Close`, size |
| [Application](app/README.md) | Full `Application.*` reference |
| [Types](app/types.md) | Handles, `Rect`, `Command` constants, menu/status records |
| [Controls](app/controls.md) | Buttons, text fields, lists, check boxes, radio buttons, menu bar, status line |
| [Dialogs and windows](app/modals.md) | `Dialog`, `Window`, and child attachment |
| [File dialog](app/file-dialog.md) | Modal `Application.RunFileDialog` |
| [Handlers](app/handlers.md) | `Application.OnCommand`, `OnKey`, `OnMouse` |
| [Lifecycle](app/lifecycle.md) | Open, run, quit, close, shared live session |
| [Native testing](app/testing.md) | Headless tests with `OpenForTest` and Turbo Vision `Test*` helpers |
| [VM bridge](app/vm-bridge.md) | Pascal-to-intrinsic map for contributors |
| [Terminal checklist](terminal-checklist.md) | Local verification commands |
| [Cell width](cell-width.md) | Unicode display-width policy |

## See Also

- [`Std.Console`](../console/README.md)
- [Std index](../README.md)
