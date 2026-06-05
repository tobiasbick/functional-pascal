# `Std.Graph` — dispatch-mode application

**Status:** current specification for the Rust-hosted event loop and `On*` handlers for native window programs.

**Maintenance (implementers only):** keep this file aligned with [`loaded/graph/`](../../../crates/fpas-sema/src/std_registry/loaded/graph/mod.rs) (see root [AGENTS.md](../../../AGENTS.md)).

---

## Model

Full graph programs use **hosted dispatch**, not manual event polling:

```pascal
var App := Application.Open(Width, Height, Title);
Application.Configure(App, Handlers);
Application.Run(App)
```

The Rust host owns the blocking loop. It waits for native input (Winit), coalesces resize bursts, invokes registered `On*` handlers, and presents frames after `OnPaint`.

Poll-style `Application.PollEvent` / `Application.ReadEventTimeout` **do not exist** on `Std.Graph`.

---

## `ApplicationHandlers`

| Field | Required | Signature |
|-------|----------|-----------|
| `OnPaint` | yes | `procedure (Application)` |
| `OnKeyPressed` | no | `function (Application, Std.Console.KeyEvent): boolean` |
| `OnMouse` | no | `procedure (Application, Event)` |
| `OnWheel` | no | `procedure (Application, Event)` |
| `OnResize` | no | `procedure (Application, Size)` |
| `OnCloseRequested` | no | `procedure (Application)` |
| `OnIdleMilliseconds` | no | `integer` (`<= 0` disables idle) |
| `OnIdle` | no | `procedure (Application)` |
| `OnExit` | no | `procedure (Application, ExitReason)` |

Register once with `Application.Configure(App, Handlers)` or the explicit `Application.HostRegisterOn*` helpers.

---

## Lifecycle

| Call | Role |
|------|------|
| `Application.Open(Width, Height, Title)` | Open one native window |
| `Application.Configure(App, Handlers)` | Install hosted handlers |
| `Application.Run(App)` | Block until quit / window close / host stop; auto-close on exit |
| `Application.RequestRedraw(App)` | Mark frame dirty; host invokes `OnPaint` on next loop turn |
| `Application.HostRequestQuit(App)` | Cooperative quit from a handler |
| `Application.Close(App)` | Close session; during `Run`, requests structured host stop |

Drawing helpers (`Clear`, `DrawLine`, `Present`, …) remain available inside `OnPaint` and other handlers. The host calls `Present` automatically after `OnPaint` during hosted redraw.

---

## `ExitReason`

Variants: `UserQuit`, `WindowClosed`, `HostStop`, `HostAndUserStop`, `HostShutdown`.

- `UserQuit` — `Application.HostRequestQuit` ended the run
- `WindowClosed` — native close request (`CloseRequested`) ended the run
- `HostStop` — `Application.Close` during an active run
- `HostAndUserStop` — both quit and host stop in the same turn
- `HostShutdown` — VM shutdown during `Run`

---

## VM bridge (Graph intrinsics)

Hosted Graph intrinsics use discriminants **331–342** (see [`graph.rs`](../../../crates/fpas-bytecode/src/intrinsic/graph.rs)). Core entrypoints:

| Pascal | Intrinsic |
|--------|-----------|
| `Application.Configure` | `ApplicationConfigure` (296) |
| `Application.Run` | `ApplicationRun` (331) |
| `Application.HostRequestQuit` | `HostRequestQuit` (332) |
| `Application.HostProcessNext` | `HostProcessNext` (335) |
| `Application.HostDispatchRedraw` | `HostDispatchRedraw` (337) |

Shared internal event normalization lives in [`fpas-std/src/ui/`](../../../crates/fpas-std/src/ui/mod.rs) (`UiHost`, `UiEvent`).

---

## Example

See [`examples/pascal/std/graph_basics.fpas`](../../../examples/pascal/std/graph_basics.fpas) and [`examples/math/julia/julia_graph.fpas`](../../../examples/math/julia/julia_graph.fpas).
