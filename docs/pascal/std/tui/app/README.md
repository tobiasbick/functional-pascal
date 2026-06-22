# `Std.Tui` — dispatch-mode application

**Status:** current specification for the Rust-hosted event loop and `On*` handlers described in [TUI framework](../../../../future/tui-application-framework.md). The host now uses a retained view engine with composable absolute transforms, effective ancestor clips, depth-first painting, state-derived focus paths, typed event routes, pointer capture, sourced internal commands, and typed process outcomes. **`Application.Host*`** dispatch helpers remain the registered Pascal bridge, **`ApplicationHandlers`** / **`Application.Configure(App, Handlers)`** provide bundled registration, and **`Application.Run(App)`** is the hosted loop entrypoint. Session handle APIs: [Session API](../session.md).

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

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Sema registry | [`loaded/tui/mod.rs`](../../../../../crates/fpas-sema/src/std_registry/loaded/tui/mod.rs) |
| Retained view engine | [`fpas-std/src/tui/view/`](../../../../../crates/fpas-std/src/tui/view/) |
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
- [TUI framework](../../../../future/tui-application-framework.md)
- [Standard library index](../../README.md)
