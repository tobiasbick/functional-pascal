# Effects in FPAS

This deferred proposal starts with an FPAS library pilot for composing effectful
workflows. Static effect checking is a separate language decision.

## What an effect system tracks

A type-and-effect system describes both the value an expression can return and
the effects its evaluation may perform. Effects can include I/O, changes to
shared state, and exceptions. The effect information can overapproximate what
actually happens on a particular run. A compiler can use it to enforce rules
such as which operations a pure function may call.
[Wikipedia overview](https://en.wikipedia.org/wiki/Effect_system) ·
[Lucassen and Gifford, Polymorphic Effect Systems](https://doi.org/10.1145/73560.73564)

FPAS functions can currently perform I/O, mutate objects, and throw CLR
exceptions. `await using` cleans up resources at lexical scope exit. Callers
pass cancellation tokens and explicitly await the tasks they own.
[Function effects](../pascal/language/functions/scope-and-effects.md) ·
[Task ownership](../pascal/agents/concurrency.md#ownership-and-shared-state)

## Library pilot

[doc-examples](../../apps/doc-examples/src/processes.fpas) and
[Measure](../../apps/diagnostics/Measure/processes.fpas) both set process
deadlines, drain stdout and stderr, and join work during cleanup. Start with one
compile-and-run workflow in `doc-examples`. Keep process ownership with
`OwnedProcess` and use the library to coordinate the work around it.

- Represent a deferred operation as a generic record. Store a callable that
  accepts a typed environment and a `CancellationToken` and returns
  `Task<Result<A, E>>`. `Run` invokes the callable. The result is an FPAS variant
  with cases for success and expected failure. The pilot uses current syntax.
- Start with `Map`, `AndThen`, and `Run`. Use a concrete environment record and
  a named error variant for the workflow. FPAS generic inference works with the
  types supplied by callers. When steps use different error or environment
  types, map the errors or project the environment explicitly.
  [Generic inference](../pascal/language/functions/type-inference.md)
- Return expected domain failures as data. Preserve cancellation and unexpected
  CLR exceptions as distinct outcomes at execution. Decide how to preserve both
  failures if the operation and its cleanup each fail.
- A scope that starts child tasks should request cancellation of unfinished
  children on exit and await them before releasing resources they still use.
  A timeout requests cancellation. Calls that ignore the token can delay scope
  exit.

Check that building the workflow leaves the process untouched until `Run`, that
an expected failure skips later steps, and that completion and cancellation
both join the output readers and release the process. Use a substitute process
operation to test sequencing and error handling deterministically. Use the real
process test to check lifetime and cleanup.

Try the same core in `Measure` before extracting a shared library. Compare the
result with the existing `async` and `await using` code. Extract the library if
both call sites carry less cancellation and cleanup logic and their tests can
exercise the workflow with fewer real process launches.

## Library contracts

- A layer builds a service from its typed dependencies. Compose layers to supply
  the environment for a workflow. Acquire dependencies before their consumers
  and release them after those consumers. If acquisition fails partway through,
  release the resources already acquired.
- A scope owns child scopes, tasks, and cleanup actions. A child scope can close
  while its parent and siblings remain active. Closing a scope requests
  cancellation in that scope and its children. Wait for tasks to finish before
  releasing resources they use. Each registered cleanup runs once, including
  after failure or cancellation. Callers can also cancel and await an individual
  task.
- An event stream preserves the producer's event order and distinguishes normal
  completion, producer failure, and cancellation. A reader that continues reading
  receives buffered events before it sees a producer failure. Cancellation must
  wake readers waiting for data and producers waiting for space. Use the existing
  [bounded channel and task ownership patterns](../pascal/libraries/channels.md).

## Test producer

After the process pilot, run a test producer in a child scope. Give it a resource
from a layer and have it write a known sequence to a bounded channel. Coordinate
the producer and reader with explicit signals so the test controls when either
can proceed.

- On success, the reader receives the full sequence in order and sees completion.
- On producer failure, the reader receives the buffered events in order before
  it sees the failure.
- For cancellation, close the child scope with a blocked producer in one run and
  a blocked reader in another. Both runs unblock and join the tasks. The parent
  scope and a sibling scope remain usable.

For each outcome, check that the resource is released once after the tasks finish.
Also check that a layer acquisition failure releases any resources acquired
earlier in the same build.

## Compiler questions

An effect value gives callers a way to compose work and choose when to run it.
Its types describe the dependencies and expected failures that the library
exposes. Compiler-checked effects would apply rules to the operations inside
ordinary FPAS routines and delegates.

- Decide which effects callers need to distinguish. Start with the guarantees
  required by the [pure-functions proposal](language-gaps.md#deferred-proposal-compiler-checked-pure-functions).
- Decide how effects compose through calls and branches, and how a routine
  that invokes a callback carries the callback's effects. Determine where the
  compiler can infer effects and where a public declaration needs an annotation.
- Define which imported .NET calls the compiler may trust and how to classify
  calls whose effects are unknown. Preserve checked guarantees across FPAS
  assemblies, delegates, and protocol dispatch.

Use the process pilot and the test producer to identify guarantees that need
compiler checking. Syntax decisions require separate approval.
