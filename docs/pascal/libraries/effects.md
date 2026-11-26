# Deferred effects

Use `Fpas.Execution.Effects` to describe work with a typed result, expected failure and
dependencies. `Fpas.NET.Sdk` supplies the assembly to every FPAS project. Import
it with `uses Fpas.Execution.Effects as Fx;`.

`Fx.Effect of (A, E, R)` is a generic record. `A` is the success value, `E` is an
expected failure and `R` is the environment required to run the work. Use a
record for multiple dependencies and a variant for distinct expected failures.
Use `Fx.Completion` as the success type when there is no value to return.

1. Pass a function taking `R` and a cancellation token to `Fx.Effect of (A, E, R).Defer`.
   The function returns `Task of Fx.Outcome of (A, E)`. Creating or composing the effect
   stores work for later.
2. Compose with `Fx.Map` to transform a success or `Fx.AndThen` to choose the next
   effect after success. Use `Fx.MapFailure` to change the failure type,
   `Fx.OrElse` for recovery and `Fx.FromEnvironment` to project dependencies.
3. Call `Run(Environment, Cancellation)` at the execution boundary. Each call
   invokes the body again. Match `Fx.Outcome.Success(Value)` and
   `Fx.Outcome.Failure(Reason)` to use the result.

An effect body uses ordinary FPAS and .NET APIs. Translate known operational
failures into `Outcome.Failure`. An unexpected CLR exception faults the task,
and cooperative cancellation follows .NET task cancellation rules.

## I/O examples

- [HTTP](../../../examples/io/http-effect/README.md) supplies a reusable
  `HttpClient` as the environment, forwards the request token, and disposes
  each response. An HTTP status and a request failure have distinct cases.
- [SQLite](../../../examples/io/sqlite-effect/README.md) supplies a connection
  string, opens and disposes a connection per run, and distinguishes a missing
  row from a provider failure.

[Workflows](workflows.md) adds scoped dependencies, retry, deadlines, parallel
work and streams on top of this type. The [MVU runtime](../../ui/mvu/runtime.md#background-effects)
executes the same effects and maps their outcomes to application messages. See
the [desktop commands example](../../../examples/ui/gui/desktop-commands/README.md#background-progress)
for an effect that reports progress as it runs.

## See also

- [Library guides](README.md)
- [HTTP request lifetime and cancellation](http.md)
- [Task coordination](task-coordination.md)

## Source files

- [Effect.fpas](../../../src/Fpas.Execution.Effects/Effect.fpas)
- [EffectExamplesTests.cs](../../../tests/Fpas.IntegrationTests/Projects/EffectExamplesTests.cs)
