# Deferred workflows

Use `Fpas.Workflows` when an application needs to configure, compose, or repeat
asynchronous work before it runs. Add a project reference to
[Fpas.Workflows.fpasproj](../../../src/Fpas.Workflows/Fpas.Workflows.fpasproj).
The library uses FPAS records and variants and runs on .NET tasks.

## Describe and run a workflow

`Workflow<A, E, R>` names the success type `A`, expected failure type `E`,
and environment `R`. Use a record for several dependencies and a variant for
different expected failures.
These types check composition at the library interface. A workflow body can use
ordinary FPAS operations and CLR APIs.

1. Wrap a routine taking the environment and a cancellation token with
   `Workflow<A, E, R>.Defer`.
2. Compose it with `Map` to transform a success or `AndThen` to choose the next
   step from a successful result. `MapFailure` adapts an error type.
   `FromEnvironment` projects a smaller environment from the caller's record.
3. Call `Run(Environment, Cancellation)` at the application boundary. Each call
   invokes the workflow body again. Match `Outcome.Success(Value)` and
   `Outcome.Failure(Reason)` to handle its result.

`AndThen` skips the next step after an expected failure. Use `OrElse` for
fallback and `Retry` for a bounded number of attempts with a delay chosen from
the failure and attempt number. Retry repeats the operation, so reserve it for
work that can safely run again. Unexpected CLR exceptions fault the task.
Cancellation uses `OperationCanceledException` and is cooperative.
`WithDeadline` requests cancellation, waits for cleanup, and can report expiry
as a typed failure. Supply a .NET `TimeProvider` to time-based policies.

## Scope work and dependencies

`Scope.Use` creates a lifetime linked to a caller token. Register cleanup with
`OnClose` and start child tasks with `Fork`. Cancelling a child affects its own
subtree. A closing scope requests cancellation, joins its children, and runs
finalizers in reverse registration order, including after faults. Work that
ignores cancellation can delay closure. Treat scope handles and acquired
dependencies as valid only while the owning scope is open.

An independent task fault and cleanup faults are reported in
`AggregateException.InnerExceptions`. If the body completed and cleanup
faulted, `Data['Fpas.Workflows.CompletedValue']` carries the completed value,
including an expected failure when the value is an `Outcome`.

`Layer.Create` couples a dependency acquisition with its release. Compose
dependent acquisitions with `ThenBuild`, and run a consumer with `Provide`.
Each run acquires its own dependencies and releases them in reverse order.
If acquisition stops partway through, the scope releases previously acquired
dependencies.

## Parallel work and streams

`Parallel.All` runs at most the requested number of workflows at once and
returns success values in input order. The first observed expected failure
requests cancellation of siblings. The group joins its children before
reporting the outcome. Concurrent faults, including cleanup faults, can fault
the group. Dependencies shared by concurrent work must support that use.

`Stream.Create` describes a producer that awaits each publication.
`Consume` runs it through a fresh bounded channel and awaits each subscriber
handler in publication order. If a producer finishes or fails, the consumer
drains buffered events before observing the producer's result. Cancelling a
subscription interrupts pending reads and writes and joins the producer before
releasing dependencies. Handlers and producers should pass the supplied token
to their own asynchronous work.

## Tracing

`Trace` wraps one execution in a .NET `Activity` when the supplied
`ActivitySource` has a listener. Place the wrapper outside `Provide` to capture
acquisition through cleanup or inside `Retry` for one span per attempt.
The wrapper records completion, failure, or caller cancellation without
attaching environment or result payloads.

## Examples

The [process capture adapter](../../../src/Fpas.Processes.Workflows/Capture.fpas)
combines a process layer, concurrent readers, a byte limit per stream, and a
deadline. The [documentation checker](../../../apps/doc-examples/src/validation.fpas)
uses it to compile and execute examples. [Measure](../../../apps/diagnostics/Measure/main.fpas)
reuses the workflow for each sample. The [contract programs](../../../tests/Fixtures/workflows)
exercise composition, dependency cleanup, and bounded streams.

## See also

- [Library guides](README.md)
- [Function contracts](../language/functions/contracts.md)
- [Task coordination](task-coordination.md)
- [Bounded channels](channels.md)
