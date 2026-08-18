# Lifecycle

| Call | Role |
|------|------|
| `Application.Open(Width, Height, Title)` | Open one native window |
| `Application.Configure(App, Handlers)` | Install hosted handlers |
| `Application.Run(App)` | Block until quit / window close / host stop; auto-close on exit |
| `Application.RequestRedraw(App)` | Mark frame dirty; host invokes `OnPaint` on next loop turn |
| `Application.HostRequestQuit(App)` | Cooperative quit from a handler |
| `Application.Close(App)` | Close session; during `Run`, requests structured host stop |

Drawing helpers (`Clear`, `DrawLine`, `Present`, …) remain available inside `OnPaint` and other handlers. The host calls `Present` automatically after `OnPaint` during hosted redraw.

Explicit `Application.Close` is recommended. If execution tears down an open
application, the runtime still detaches and closes its backend best-effort so a
later VM session on the same worker thread can open a fresh application. A host
close error is reported for an explicit close, but the detached application is
already considered closed.

### Native test lifecycle

Headless graph tests use in-program APIs instead of `*.script.toml` graph events. See [`test.md`](../../testing/test.md).

| Call | Role |
|------|------|
| `Application.OpenForTest(Width, Height)` | Open a deterministic headless session (no native window) |
| `Application.TestSendKey(App, Key)` | Enqueue one `Std.Console.KeyEvent` for the next hosted pump |
| `Application.Run(App)` | Pump events and paint; auto-closes on exit (restores native backend) |

Golden pixel checks (`*.expect.pixels`) run runner-side after `Present` inside `OnPaint`.

---

## `ExitReason`

Variants: `UserQuit`, `WindowClosed`, `HostStop`, `HostAndUserStop`, `HostShutdown`.

- `UserQuit` — `Application.HostRequestQuit` ended the run
- `WindowClosed` — native close request (`CloseRequested`) ended the run
- `HostStop` — `Application.Close` during an active run
- `HostAndUserStop` — both quit and host stop in the same turn
- `HostShutdown` — VM shutdown during `Run`

## Example

See [`examples/pascal/std/graph_basics.fpas`](../../../../../examples/pascal/std/graph_basics.fpas).

## See also

- [Handlers](handlers.md)
- [VM bridge](vm-bridge.md)
- [Hosted dispatch overview](README.md)
