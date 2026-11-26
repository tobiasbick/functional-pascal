# Deferred workflows

Use `Fpas.Execution.Workflows` for execution policies and resource lifetimes on top of
the SDK's [`Effect of (A, E, R)`](effects.md). Add a project reference to
[Fpas.Execution.Workflows.fpasproj](../../../src/Fpas.Execution.Workflows/Fpas.Execution.Workflows.fpasproj).

## Retry and deadlines

`Retry` runs an effect with a bounded number of attempts and a delay chosen
from the failure and attempt number. Apply it to work that can safely run again.
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
faulted, `Data['Fpas.Execution.Workflows.CompletedValue']` carries the completed value,
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

## Capturing a child process

`Fpas.Processes.Workflows.Capture.Read(TimeoutSeconds, Limit)` builds a
deferred workflow. Supply an absolute executable path, individual arguments,
the working directory and optional environment overrides in the request.
The adapter closes the child's input and drains output and error concurrently.
A successful result contains both UTF-8 strings and the child's exit code,
including nonzero exit codes.

`Limit` bounds raw bytes **per stream**. Exceeding it faults the task with
`InvalidDataException`. The deadline produces `CaptureFailure.Deadline`;
caller cancellation cancels the task. Cleanup also terminates
[owned descendants](processes.md#owning-a-process-tree) that still hold pipes.

## Examples

The [documentation checker](../../../apps/doc-examples/src/validation.fpas)
uses it to compile and execute examples. [Measure](../../../apps/diagnostics/Measure/main.fpas)
reuses the workflow for each sample. The [contract programs](../../../tests/Fixtures/workflows)
exercise composition, dependency cleanup, and bounded streams.

## See also

- [Library guides](README.md)
- [Deferred effects](effects.md)
- [Function contracts](../language/functions/contracts.md)
- [Task coordination](task-coordination.md)
- [Bounded channels](channels.md)

## Source files

<details>
<summary>Browse 14 source files</summary>

- `src/Fpas.Processes.Workflows/`
  - [Capture.fpas](../../../src/Fpas.Processes.Workflows/Capture.fpas)
  - [Fpas.Processes.Workflows.fpasproj](../../../src/Fpas.Processes.Workflows/Fpas.Processes.Workflows.fpasproj)
- `src/Fpas.Execution.Workflows/`
  - [Layer.fpas](../../../src/Fpas.Execution.Workflows/Layer.fpas)
  - [Parallel.fpas](../../../src/Fpas.Execution.Workflows/Parallel.fpas)
  - [Policies.fpas](../../../src/Fpas.Execution.Workflows/Policies.fpas)
  - [Scope.fpas](../../../src/Fpas.Execution.Workflows/Scope.fpas)
  - [Stream.fpas](../../../src/Fpas.Execution.Workflows/Stream.fpas)
  - [Tracing.fpas](../../../src/Fpas.Execution.Workflows/Tracing.fpas)
- `tests/Fixtures/workflows/`
  - [CaptureEncoding.fpas](../../../tests/Fixtures/workflows/CaptureEncoding.fpas)
  - [Lifetimes.fpas](../../../tests/Fixtures/workflows/Lifetimes.fpas)
  - [Operations.fpas](../../../tests/Fixtures/workflows/Operations.fpas)
  - [Streams.fpas](../../../tests/Fixtures/workflows/Streams.fpas)
- `tests/Fpas.IntegrationTests/Build/`
  - [WorkflowProjects.cs](../../../tests/Fpas.IntegrationTests/Build/WorkflowProjects.cs)
- `tests/Fpas.IntegrationTests/Execution/`
  - [WorkflowTests.cs](../../../tests/Fpas.IntegrationTests/Execution/WorkflowTests.cs)

</details>
