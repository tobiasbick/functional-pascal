# Fpas.Ui.Mvu

Fpas.Ui.Mvu runs typed model and message transitions for .NET 10 hosts. It is
implemented in FPAS and exposes CLR types for use from FPAS, C# and F#.

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
view. Mapping does not start work or invoke action factories. Workers retain
their cancellation token, awaited Post task and failure-to-message mapper.

Host-specific payloads implement `Application.HostExtension<TMsg>`. Its generic
`Map<TNext>` method rebuilds the payload with translated message factories and
returns `HostExtension<TNext>`. A mapped opaque payload is rejected instead of
being delivered under the wrong message type. GUI toast requests implement this
contract, including their Undo/action callbacks.

Use [`Fpas.Ui.Composition.Map`](../Fpas.Ui/README.md#subview-composition) for
element trees. The [gallery router](../../examples/ui/gui/gallery/App/Update.fpas)
and [Actions slice](../../examples/ui/gui/gallery/Pages/Actions/Update.fpas)
show model ownership, scoped effects and message routing in one session.

Use `Fpas.Ui.Elements.CachedSubview` with only the inputs that affect that
subtree. It avoids rebuilding an unchanged view, but the hosts still validate
and process the resolved tree. Slicing alone does not provide incremental layout
or data virtualization. GUI collections process all supplied items; the TUI
limits row drawing to the viewport but still traverses complete data elsewhere.
Large collections therefore need separate virtualization work. Functional tests
and the galleries do not establish large-application scalability; measure a
representative workload before choosing further optimizations.

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

The [terminal host](../Fpas.Ui.Tui/README.md) uses this runtime for interactive
and headless sessions. The [effect contract](../../tests/Fixtures/tui/EffectContract.fpas)
drives the same update function directly and through the terminal adapter.
The [GUI host](../Fpas.Ui.Gui/README.md) also uses this runtime for shared commands,
native event wakeup and responsive asynchronous cleanup.
