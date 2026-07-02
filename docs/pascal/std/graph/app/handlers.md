# Handlers

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

Register once with `Application.Configure(App, Handlers)`.

## Example

See [`examples/pascal/std/graph_basics.fpas`](../../../../../examples/pascal/std/graph_basics.fpas)
for a minimal `OnPaint` handler wired through `Application.Configure`.

## See also

- [Hosted dispatch overview](README.md)
- [Lifecycle](lifecycle.md)
- [Session API](../session.md)
