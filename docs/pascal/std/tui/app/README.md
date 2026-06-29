# `Std.Tui` — dispatch-mode application

**Status:** current specification for the Rust-hosted event loop and `On*` handlers. The host uses a retained view engine with composable absolute transforms, effective ancestor clips, depth-first painting, state-derived focus paths, typed event routes, pointer capture, sourced internal commands, and typed process outcomes. **`Application.Host*`** dispatch helpers remain the registered Pascal bridge, **`ApplicationHandlers`** / **`Application.Configure(App, Handlers)`** provide bundled registration, and **`Application.Run(App)`** is the hosted loop entrypoint. Session handle APIs: [Session API](../session.md).

| Topic | Description |
|-------|-------------|
| [VM bridge](vm-bridge.md) | Intrinsics, modals, views, host widgets |
| [Views and focus](views.md) | Retained tree, clipping, Tab traversal, paint order |
| [Modals and dialogs](modals.md) | `ShowModal`, `ShowDialog`, results, focus restore |
| [Retained controls](controls.md) | Labels, buttons, input lines, checkboxes, radio groups |
| [Frame roots](frames.md) | Painted frames, inner viewport clipping, window management, owned framed dialogs |
| [Lifecycle](lifecycle.md) | `Open`, `Configure`, `Run`, `Close` |
| [Handlers](handlers.md) | `On*` callbacks and registration |
| [Types](types.md) | `ApplicationHandlers`, `ExitReason`, `ViewId`, signatures |
| [Native testing](testing.md) | `OpenForTest`, `TestPump`, `Query*` |

## Turbo Vision spike API

The branch-local Turbo Vision rewrite exposes a minimal headless command path for testing the new handle model. These calls create Turbo Vision-backed dialog and button handles, queue a test click, and pump one queued command through `OnCommand`.

| Symbol | Purpose |
|--------|---------|
| `TuiDialog` | Opaque dialog handle returned by `Application.CreateDialog`. |
| `TuiButton` | Opaque button handle returned by `Application.CreateButton`. |
| `Application.CreateDialog(App, Bounds, Title): TuiDialog` | Create a Turbo Vision dialog handle. |
| `Application.CreateButton(App, Bounds, Text, CommandId): TuiButton` | Create a Turbo Vision button handle. |
| `Application.AddChild(App, Dialog, Button)` | Attach a button to a dialog. |
| `Application.OnCommand(App, Handler)` | Register `procedure (Application, integer)`. |
| `Application.TestClickButton(App, Button)` | Queue the button command for a headless test. |
| `Application.Pump(App): integer` | Dispatch one queued Turbo Vision command; command handled returns `16`, idle returns `0`. |
| `Application.Quit(App)` | Request the Turbo Vision spike pump to stop dispatching further queued commands. |

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Sema registry | [`loaded/tui/mod.rs`](../../../../../crates/fpas-sema/src/std_registry/loaded/tui/mod.rs) |
| Retained view engine | [`fpas-std/src/tui/view/`](../../../../../crates/fpas-std/src/tui/view/) |
| Turbo Vision spike bridge | [`fpas-vm/.../tui/turbo_vision/`](../../../../../crates/fpas-vm/src/vm/execute/io/tui/turbo_vision/) |
| Modal stack | [`fpas-std/src/tui/modal/`](../../../../../crates/fpas-std/src/tui/modal/) |
| Host widgets | [`fpas-std/src/tui/widget/`](../../../../../crates/fpas-std/src/tui/widget/) |
| VM bridge | [`fpas-vm/.../tui/`](../../../../../crates/fpas-vm/src/vm/execute/io/tui/) |
| Contributor guide | [AGENTS.md](../../../../../AGENTS.md) |

Dialog controls (`LabelWidget`, `ButtonWidget`, `InputLineWidget`, checkbox/radio) live under [`widget/control/`](../../../../../crates/fpas-std/src/tui/widget/control/); their public Pascal bridge is documented in [Retained controls](controls.md).

## See also

- [Session API](../session.md)
- [Views and focus](views.md)
- [Modals and dialogs](modals.md)
- [`Std.Console`](../../console/README.md)
- [Terminal UI index](../README.md)
- [Standard library index](../../README.md)
