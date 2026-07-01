# Terminal UI

Terminal UI APIs for Functional Pascal.

## Two application models

`Std.Tui` exposes **two ways to build a terminal application**. Choose one model per program. **Mixing them is unsupported.**

| Model | Best for | How you build the UI | Event loop |
| --- | --- | --- | --- |
| **Turbo Vision facade** | Dialogs, windows, menus, buttons, file pickers, IDE-style chrome | Retained widget handles (`Application.CreateDialog`, `CreateButton`, `AddChild`, …) | Turbo Vision event pump; user actions arrive as integer commands via `Application.OnCommand` |
| **Hosted canvas** | Custom fullscreen rendering, animations, demos that own every pixel | `Std.Console` drawing inside an `OnPaint` handler registered through `Application.Configure` | Hosted global-handler loop (`OnPaint`, `OnKeyPressed`, `OnResize`, …) |

### How `Application.Run` chooses

`Application.Run` inspects the active session:

1. **Turbo Vision path** — runs when **any** Turbo Vision widget handle was created (`Application.CreateDialog`, `CreateWindow`, `CreateButton`, `CreateMenuBar`, and the other `Create*` calls). `OnPaint` is not used; commands flow through `Application.OnCommand`.
2. **Hosted canvas path** — runs when **no** Turbo Vision handles exist. `Application.Run` **requires** a registered `OnPaint` handler.

Creating even one Turbo Vision handle permanently selects the Turbo Vision path for that session. Calling `Application.Configure` alongside widget construction does not combine the two models — hosted handlers are ignored once widgets exist.

### Examples

| Style | Starting points |
| --- | --- |
| Turbo Vision | `examples/pascal/tui/turbo_vision_dialog.fpas`, `apps/ide` |
| Hosted canvas | `examples/pascal/tui/minimal_application.fpas`, `examples/math/mandelbrot` |

| Topic | Description |
| --- | --- |
| [Session API](session.md) | `Application.Open`, `Run`, `Close`, size, redraw |
| [Application](app/README.md) | Full `Application.*` reference |
| [Types](app/types.md) | Handles, `Rect`, `Command` constants, menu/status records |
| [Controls](app/controls.md) | Buttons, text fields, lists, check boxes, radio buttons, menu bar, status line |
| [Dialogs and windows](app/modals.md) | `Dialog`, `Window`, and child attachment |
| [File dialog](app/file-dialog.md) | Modal `Application.RunFileDialog` |
| [Handlers](app/handlers.md) | `ApplicationHandlers`, `Application.OnCommand` |
| [Lifecycle](app/lifecycle.md) | Open, run, quit, and close rules |
| [Native testing](app/testing.md) | Headless tests with `OpenForTest` and `Test*` helpers |
| [VM bridge](app/vm-bridge.md) | Pascal-to-intrinsic map for contributors |
| [Terminal checklist](terminal-checklist.md) | Local verification commands |
| [Cell width](cell-width.md) | Unicode display-width policy |

## See Also

- [`Std.Console`](../console/README.md)
- [Std index](../README.md)
