# Asynchronous execution and concurrency

## Async declarations

```pascal
async function Download(Address: string;
  Cancellation: Threading.CancellationToken): Tasks.Task<string>;
begin
  using const Client: Http.HttpClient := new Http.HttpClient();
  return await Client.GetStringAsync(Address, Cancellation);
end function;
```

`async` marks a function and its body. The declared return type is explicitly
`System.Threading.Tasks.Task<T>` or non-generic `Task`. A `return Value;` inside
`Task<T>` supplies `T`; `return;` completes a non-generic Task. Every normal path
of an async `Task<T>` function must return a value. Async functions returning
non-generic Task may complete at the end of their body.

There are no async procedures/async void functions. An anonymous async function
uses `async function(...): Task<T> begin ... end function`. Async expression
lambdas use `async function(...): Task<T> => Expression` and supply a `T` result.
For an ordinary synchronous function returning Task, normal return rules apply:
it returns the task object itself and cannot use await.

Contracts are permitted on async functions. Requires checks run before the body
inside the state machine; ensures checks run after successful cleanup and bind
the inner T result rather than the Task<T>. A failed check faults the Task;
fault/cancellation skips postconditions. See [contracts](contracts.md#async-routines).

Calling an async function begins execution synchronously until its first
incomplete await. It does not automatically start a thread or queue CPU work.
Task results may be awaited multiple times under normal .NET rules.

## Await

`await Expression` is valid in an async function/lambda or an async program body.
It suspends without blocking the executing thread when completion is pending.
Awaiting `Task<T>` yields `T`; awaiting Task has no value and is a statement.
Awaiting a faulted/canceled operation propagates the corresponding .NET exception.

The baseline consumes Task, Task<T>, ValueTask, ValueTask<T>, and their standard
ConfigureAwait awaitables. It does not produce ValueTask from an async declaration
or support arbitrary custom awaiter patterns. ValueTask consumption must respect
its producer's .NET lifetime/reuse rules; it is not assumed reusable like Task.

Default continuation-context behavior matches ordinary .NET await. Call
`ConfigureAwait(false)` explicitly when appropriate. Await does not imply a new
synchronization context, thread affinity, or mutual exclusion.

## Entry point

```pascal
async program AsyncHello;

uses System.Console as Console, System.Threading.Tasks as Tasks;

begin
  await Tasks.Task.Delay(10);
  Console.WriteLine('Ready');
end;
end program;
```

The compiler generates the executable entry-point bridge and waits for the
program task before process completion. A plain `program` cannot await.
User functions do not receive an implicit blocking bridge.

## Cancellation, parallel work, and channels

Cancellation is explicit through `System.Threading.CancellationToken` and
`CancellationTokenSource`. Accept and forward tokens to cancelable operations;
await does not inject a token or make an API cancelable.

Use Task.Run for appropriate CPU-bound work, Task.WhenAll/WhenAny for coordination,
and System.Threading.Channels for bounded or unbounded channels. These are normal
.NET APIs, not syntax or FPAS wrappers. Task.Run overload resolution uses the
ordinary delegate rules and .NET's task-unwrapping overloads.

There is no `go`, special task type, task ownership scope, transferability check,
or implicit cancellation/join at a function boundary. Explicitly await work whose
completion belongs to the operation. Exceptions from abandoned tasks are not
automatically observed. Await uses the task's ordinary .NET fault/cancellation
behavior; the compiler does not insert a separate error wrapper.

Shared object state and captured `var` bindings have ordinary CLR behavior.
The language does not make data races impossible. Use actual .NET synchronization
APIs or immutable data; there is no special language scheduler or monitor syntax
in this baseline. Byref parameters and byref-like values cannot cross await.

Structured task groups could be a library feature, but are not an implicit
language guarantee. No detached execution, channel selection, supervised retry,
or timeout policy is invented by the compiler.
