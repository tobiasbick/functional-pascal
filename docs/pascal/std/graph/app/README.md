# `Std.Graph` — dispatch-mode application

**Status:** current specification for the Rust-hosted event loop and `On*` handlers for native window programs.

## Model

Full graph programs use **hosted dispatch**, not manual event polling:

```pascal
var App := Application.Open(Width, Height, Title);
Application.Configure(App, Handlers);
Application.Run(App)
```

The Rust host owns the blocking loop. It waits for native input (Winit), coalesces resize bursts, invokes registered `On*` handlers, and presents frames after `OnPaint`.

Poll-style `Application.PollEvent` / `Application.ReadEventTimeout` **do not exist** on `Std.Graph`.

| Topic | Description |
|-------|-------------|
| [Handlers](handlers.md) | `ApplicationHandlers` record |
| [Lifecycle](lifecycle.md) | `Open`, `Configure`, `Run`, `ExitReason` |
| [VM bridge](vm-bridge.md) | Graph intrinsics and test entrypoints |

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Sema registry | [`loaded/graph/mod.rs`](../../../../../crates/fpas-sema/src/std_registry/loaded/graph/mod.rs) |
| Contributor guide | [AGENTS.md](../../../../../AGENTS.md) |

## See also

- [Session API](../session.md)
- [`Std.Test`](../../testing/test.md)
- [Graphics index](../README.md)
- [Standard library index](../../README.md)
