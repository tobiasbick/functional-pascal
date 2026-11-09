# Fpas.Mvu

Fpas.Mvu runs typed model and message transitions for .NET 10 hosts. It is
implemented in FPAS and exposes CLR types for use from FPAS, C# and F#.

## Driving a session

In `Fpas.Mvu.Application`, `Transition<TModel, TMsg>` contains the next model
and an array of `Command<TMsg>`. `Fpas.Mvu.Session.Start` takes an initial
model, an `Update` function, a host request handler and a mailbox capacity.
The host owns input routing, the view and presentation. `Focus` and `Exit`
produce host requests, so the host resolves control IDs and manages its own
resources.

Drive `Advance`, `Dispatch`, model reads, effect diagnostics and `Close` on
one owner. Keep `Update` and the host request handler short. The handler
handles the request on the owner and returns without reentering the session.
An update commits its model before its commands run. Rendering can read the
model repeatedly without applying the transition again.

`Send` admits raw host input and `Post` admits an application message. Either
can be called by concurrent producers and returns `Queued`, `Full` or `Closed`.
Each `Advance` consumes at most one FIFO entry. If it returns raw input, the
host can route it against its current view and call `Dispatch` to apply the
resulting message in that turn. `WaitForWork` and `WaitForSpace` signal possible
progress. Retry admission after a capacity signal, since another producer can
take the space first.

## Background effects

`StartBackground` runs the entire effect body on a worker, including the work
before its first `await`. The effect receives a cancellation token and a `Post`
function. Await `Post` to respect mailbox capacity. Its completion reports
admission, and `Update` receives the message on a later owner turn. The failure
mapper runs on the worker and converts an ordinary effect failure into a
message using the same delivery path.

Use stable effect IDs. Starting another effect with the same ID supersedes
the previous run. `CancelBackground` invalidates that run explicitly. Both
operations immediately invalidate queued messages from the old run and request
asynchronous cancellation. The owner can process more input while the worker
and any cancellation callbacks finish.

## Cancellation and shutdown

`Close` and the close request from `Exit` end admission, discard pending
entries and request cancellation of active effects. Waiting producers are
released. `Close` returns the same task on repeated calls. Observe that task
before disposing resources used by effects. It completes after workers and
cancellation callbacks finish and reports cleanup failures. Workers and
callbacks must finish cooperatively for shutdown to complete.

The [terminal host](../Fpas.Tui/README.md) uses this runtime for interactive
and headless sessions. The [effect contract](../../tests/Fixtures/tui/EffectContract.fpas)
drives the same update function directly and through the terminal adapter.
