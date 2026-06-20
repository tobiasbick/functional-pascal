# `Std.Tui` — dispatch-mode application

**Status:** current specification for the Rust-hosted event loop and `On*` handlers described in [TUI framework](../../../../future/tui-application-framework.md). The host now uses a retained view engine with composable absolute transforms, effective ancestor clips, depth-first painting, state-derived focus paths, typed event routes, pointer capture, sourced internal commands, and typed process outcomes. **`Application.Host*`** dispatch helpers remain the registered Pascal bridge, **`ApplicationHandlers`** / **`Application.Configure(App, Handlers)`** provide bundled registration, and **`Application.Run(App)`** is the hosted loop entrypoint. Session handle APIs: [Session API](../session.md).

| Topic | Description |
|-------|-------------|
| [VM bridge](vm-bridge.md) | Intrinsics, modals, views, host widgets |
| [Lifecycle](lifecycle.md) | `Open`, `Configure`, `Run`, `Close` |
| [Handlers](handlers.md) | `On*` callbacks and registration |
| [Types](types.md) | `ApplicationHandlers`, `ExitReason`, signatures |
| [Native testing](testing.md) | `OpenForTest`, `TestPump`, `Query*` |

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Sema registry | [`loaded/tui/mod.rs`](../../../../../crates/fpas-sema/src/std_registry/loaded/tui/mod.rs) |
| Contributor guide | [AGENTS.md](../../../../../AGENTS.md) |

## See also

- [Session API](../session.md)
- [`Std.Console`](../../console/README.md)
- [Terminal UI index](../README.md)
- [TUI framework](../../../../future/tui-application-framework.md)
- [Standard library index](../../README.md)
