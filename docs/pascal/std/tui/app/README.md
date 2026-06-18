# `Std.Tui` — dispatch-mode application

**Status:** current specification for the Rust-hosted event loop and `On*` handlers described in [TUI framework](../../../../future/tui-application-framework.md). **`Application.Host*`** dispatch helpers are **registered and lowered**, **`ApplicationHandlers`** / **`Application.Configure(App, Handlers)`** are available as the bundled registration surface, **`Application.Run(App)`** is the hosted loop entrypoint, and the current Phase 7 structure layer includes **`Std.Tui.Rect`**, **`Application.HostSetViewParent`**, **`Application.HostRegisterOnViewPaint`**, **`Application.HostBindCommandToView`**, **`Application.HostBindCommandToActiveModal`**, **`Application.ShowModal`**, **`Application.ShowDialog`**, and **`Application.CloseModal`**. `OnIdle` is available through `Application.HostRegisterOnIdle(App, Milliseconds, OnIdle)` and the bundle fields `OnIdleMilliseconds` + `OnIdle`. Session handle APIs: [Session API](../session.md).

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
