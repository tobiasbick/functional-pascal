# Async functions and lambdas

Declare a .NET Task result and await supported .NET operations inside the body.

```pascal fragment context=async-declarations
async function Download(
  Address: string;
  Cancellation: Threading.CancellationToken
): Tasks.Task of string;
begin
  using const Client: Http.HttpClient := new Http.HttpClient();
  return await Client.GetStringAsync(Address, Cancellation);
end function;
```

`async` marks a function and its body. The declared return type is explicitly
`System.Threading.Tasks.Task of T` or non-generic `Task`. A `return Value;` inside
`Task of T` supplies `T`; `return;` completes a non-generic Task. Every normal path
of an async `Task of T` function must return a value. Async functions returning
non-generic Task may complete at the end of their body.

There are no async procedures/async void functions. An anonymous async function
uses `async function(...): Task of T begin ... end function`. Async expression
lambdas use `async function(...): Task of T => Expression` and supply a `T` result.
For an ordinary synchronous function returning Task, normal return rules apply:
it returns the task object itself and cannot use await.

Contracts are permitted on async functions. Requires checks run before the body
inside the state machine; ensures checks run after successful cleanup and bind
the inner T result rather than the Task<T>. A failed check faults the Task;
fault/cancellation skips postconditions. See [contracts](../functions/contracts.md#async-routines).

Calling an async function begins execution synchronously until its first
incomplete await. It does not automatically start a thread or queue CPU work.
Task results may be awaited multiple times under normal .NET rules.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)

## Source files

- `apps/doc-examples/contexts/`
  - [async-declarations.fpas.txt](../../../../apps/doc-examples/contexts/async-declarations.fpas.txt)
- `tests/Fpas.Compiler.Tests/Metadata/`
  - [AsyncStackDepthTests.cs](../../../../tests/Fpas.Compiler.Tests/Metadata/AsyncStackDepthTests.cs)
- `tests/Fpas.Compiler.Tests/Routines/`
  - [AsyncDiagnosticsTests.cs](../../../../tests/Fpas.Compiler.Tests/Routines/AsyncDiagnosticsTests.cs)
- `tests/Fpas.IntegrationTests/Build/`
  - [MsBuildTests.Async.cs](../../../../tests/Fpas.IntegrationTests/Build/MsBuildTests.Async.cs)
- `tests/Fpas.IntegrationTests/Execution/`
  - [AsyncExecutionTests.cs](../../../../tests/Fpas.IntegrationTests/Execution/AsyncExecutionTests.cs)
