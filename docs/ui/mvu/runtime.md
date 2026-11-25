# Fpas.Ui.Mvu

Fpas.Ui.Mvu runs typed model and message transitions for .NET 10 hosts. It is
implemented in FPAS and exposes CLR types for use from FPAS, C# and F#.

For a short introduction and a feature composition example, see [Model–View–Update](README.md).

## Driving a session

In `Fpas.Ui.Mvu.Application`, `Transition<TModel, TMsg>` contains the next model
and an array of `Command<TMsg>`. `Fpas.Ui.Mvu.Session.Start` takes an initial
model, an `Update` function, a host request handler and a mailbox capacity.
The host owns input routing, the view and presentation. `Focus` and `Exit`
produce host requests, so the host resolves control IDs and manages its own
resources.
Host-specific libraries can issue `HostRequest.Extension` with a typed
payload. The receiving host defines and validates the supported payloads.

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

An optional nonthrowing `Wake` callback runs on the producer after successful
admission, including background results. Native hosts can wake their owner loop
without polling or mutating UI state from a worker.

## Larger applications

Split substantial applications into feature slices, each with its own model,
message type, update function and view. The parent routes messages to the owning
slice and combines its transition with the application state in one session.
Keep shared state under an explicit owner and control/effect IDs distinct across
slices. Use `Fpas.Ui.Mvu.Composition.MapTransition<TModel, TChild, TParent>`
to translate a child's transition with a pure message wrapper such as a parent
variant case. It retains the next child model and returns parent commands in
their original order. The parent places that child model in its own model.
`MapCommand<TChild, TParent>` translates an individual command.

The optional `Scope` prefixes background start and cancel IDs with `Scope/`.
Use the same scope for a slice's lifetime so cancellation and supersession reach
its own runs. Focus IDs stay unchanged: they identify controls in the complete
view. Mapping a command preserves deferred execution, the cancellation token,
and backpressure while translating posts and expected failures to parent messages.

Host-specific payloads implement `Application.HostExtension<TMsg>`. Its generic
`Map<TNext>` method rebuilds the payload with translated message factories and
returns `HostExtension<TNext>`. A mapped opaque payload is rejected instead of
being delivered under the wrong message type. GUI toast requests implement this
contract, including their Undo/action callbacks.

Use [`Fpas.Ui.Composition.Map`](../widgets.md#subview-composition) for
element trees. The [gallery router](../../../examples/ui/gui/gallery/App/Update.fpas)
and [Actions slice](../../../examples/ui/gui/gallery/Pages/Actions/Update.fpas)
show model ownership, scoped effects and message routing in one session.

Use `Fpas.Ui.Elements.CachedSubview` with only the inputs that affect that
subtree. It avoids rebuilding an unchanged view, but the hosts still validate
and process the resolved tree. Slicing alone does not provide incremental layout
or data virtualization. For large graphical collections, use
[`VirtualLists.VirtualList`](../widgets.md#data-display) or
[`DataGrids.DataGrid`](../widgets.md#data-grid-filter-bar-and-tree-table):
their pure row factories realize only CUI's visible range and overscan.
Ordinary collection views still process all supplied items. The TUI limits
drawing to the viewport but still traverses the complete view elsewhere.
Functional tests and the galleries do not establish large-application
scalability; measure a representative workload before choosing further
optimizations.

## Background effects

`StartBackground` runs an SDK [effect](../../pascal/libraries/effects.md) on a
worker, including the work before its first `await`. It provides a cancellation
token and `BackgroundContext<TMsg>` with `Post`. Await `Post` to respect mailbox
capacity. Its completion reports admission, and `Update` receives the message
on a later owner turn.

Use `FromEffect` to project the dependencies of an `Effect<A, E, R>` from the
running context and translate `Outcome.Success` and `Outcome.Failure` into
messages. The dependency projection can pass `Post` through for progress.
Unexpected CLR exceptions fault the worker. Use `FromWorker` to adapt a
callback that posts its own replies and maps exceptions to expected failure
messages. Both adapters produce the same effect type for `StartBackground`.
The [desktop settings example](../../../examples/ui/gui/desktop-commands/README.md#appearance-and-layout)
uses `FromEffect` for saving a value. The [podcast example](../../../apps/fpas-amp/doc/podcasts.md#implementation)
posts search and feed results before loading their artwork, and returns expected
network failures as typed messages.

Use stable effect IDs. Starting another effect with the same ID supersedes
the previous run. `CancelBackground` invalidates that run explicitly. Both
operations immediately invalidate queued messages from the old run and request
asynchronous cancellation. The owner can process more input while the worker
and any cancellation callbacks finish.

## Cancellation and shutdown

`Close` and the close request from `Exit` end admission, discard pending
entries and request cancellation of active effects. If a host request closes
the session during command dispatch, the remaining commands are skipped.
Waiting producers are released. `Close` returns the same task on repeated calls. Observe that task
before disposing resources used by effects. It completes after workers and
cancellation callbacks finish. Waiting effect posts end as cancellation, and
concurrent worker and callback faults are both available from the task. Workers and
callbacks must finish cooperatively for shutdown to complete.

The [lifetime test](../../../tests/Fpas.Ui.Mvu.Tests/LifetimeTests.fpas)
checks closing while a typed reply is waiting for mailbox capacity.

The [terminal host](../tui/host.md) uses this runtime for interactive
and headless sessions. The [effect contract](../../../tests/Fixtures/tui/EffectContract.fpas)
drives the same update function directly and through the terminal adapter.
The [GUI host](../gui/host.md) also uses this runtime for shared commands,
native event wakeup and responsive asynchronous cleanup.

## Testing effect replies

Reference [Fpas.Ui.Mvu.Testing](../../../src/Fpas.Ui.Mvu.Testing/Fpas.Ui.Mvu.Testing.fpasproj)
from a test project. Use `Effects<TMsg>.Watch(id)` before dispatching the operation
and wrap the resulting transition with `Wrap`. Read a held reply from the watcher,
assert the intermediate model, then await `Release()` and advance the session.
The [effect reply test](../../../tests/Fpas.Ui.Mvu.Tests/EffectRepliesTests.fpas)
drives supersession and typed outcomes through a real mailbox. Run it through the
[MVU test project](../../../tests/Fpas.Ui.Mvu.Tests/Fpas.Ui.Mvu.Tests.fpasproj).

## Source files

- `src/Fpas.Ui.Mvu/`
  - [Application.fpas](../../../src/Fpas.Ui.Mvu/Application.fpas)
  - [Composition.fpas](../../../src/Fpas.Ui.Mvu/Composition.fpas)
  - [Fpas.Ui.Mvu.fpasproj](../../../src/Fpas.Ui.Mvu/Fpas.Ui.Mvu.fpasproj)
  - [Session.fpas](../../../src/Fpas.Ui.Mvu/Session.fpas)
- `src/Fpas.Ui.Mvu/Internal/`
  - [Effects.fpas](../../../src/Fpas.Ui.Mvu/Internal/Effects.fpas)
- `src/Fpas.Ui.Mvu.Testing/`
  - [EffectReplies.fpas](../../../src/Fpas.Ui.Mvu.Testing/EffectReplies.fpas)
