# Lifecycle

## Relationship to session APIs

[Session API](../session.md) documents the session handle (`Application.Open`, `Application.Size`, `Application.RequestRedraw`) and the hosted entry points (`Application.Configure`, `Application.Run`).

Dispatch-mode names use the `**On` prefix** so they do not collide with legacy names such as console `**KeyPressed`** (boolean poll).

---

## Session and entry


| Step                | Meaning                                                                                                                                                               |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Application.Open`  | Same session semantics as today: acquire terminal state (raw mode, alternate screen when applicable).                                                                 |
| `Application.Run`   | Start the **hosted** main loop for the given `Application` handle. Register handlers first with `Application.Configure(App, Handlers)`, the explicit `Application.HostRegisterOn*` helpers, per-view `Application.HostRegisterOnViewPaint`, or host widget constructors (`HostCreateSolidFillView`, `HostCreateMenuBarView`, `HostCreateStatusBarView`); at least one global `OnPaint`, local view paint handler, or host widget view is required. The loop auto-requests the first redraw and blocks until the application requests quit. |
| `Application.Close` | Release the session. After `**Application.Run`** completes successfully, the host **must** have restored the session as if `**Close`** ran (see **Lifecycle** below). |

**Current Pascal surface:** `**Application.Configure(App, Handlers)`** lowers to a dedicated intrinsic and writes the bundled hosted handlers (`**OnPaint`** required in bundle form; optional handlers use `**Some(...)`** / `**None`**). `**Application.Run(App)`** then uses whichever handlers were registered last, whether through `**Configure`**, the explicit `**Application.HostRegisterOn*`** helpers, per-view `**Application.HostRegisterOnViewPaint`** registrations, or host widget views created before `**Run`**. `**TuiHostRunLoop**` (**262**) remains available as the low-level bounded stepping helper for tests and explicit host experimentation.

### Lifecycle (normative)

1. User calls `**Application.Open`** → receives `**App`**.
2. User registers handlers with `**Application.Configure(App, Handlers)`**, `**Application.HostRegisterOn*`**, optionally `**Application.HostRegisterOnViewPaint`** for individual views, and/or host widget views (`**HostCreateSolidFillView**`, `**HostCreateMenuBarView**`, `**HostCreateStatusBarView**`). A hosted run requires at least one global `**OnPaint`** handler, at least one local view paint handler, or at least one host widget view.
3. User calls `**Application.Run(App)`**.
4. While running, the host dispatches `**On*`** handlers on the **main VM thread** only (see [Scheduling](../../../language/concurrency/scheduling.md)).
5. When the application requests quit, the host records `**ExitReason.UserQuit`**. If the active hosted session is stopped by low-level host control during `**Run`** (for example `**Application.Close(App)`** is invoked while the run is still active), the host records `**ExitReason.HostStop`**. If both are requested in the same turn, the host records `**ExitReason.HostAndUserStop`**. If the VM enters global shutdown while the hosted run is active (for example after a concurrent task failure), the host records `**ExitReason.HostShutdown`**. In every case it invokes `**OnExit(App, Reason)`** once if that handler is provided, then **performs `Application.Close(App)`** (or equivalent) so the program must **not** call `**Close`** again for the same successful `**Run`**.

If `**Run`** is never called, the program keeps today’s obligation: `**Open**` / `**Close**` pairing without `**Run**`.

## See also

- [Hosted dispatch overview](README.md)
- [Handlers](handlers.md)
- [Session API](../session.md)
