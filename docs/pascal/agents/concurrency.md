# Async functions, tasks, and cancellation

Read this before starting concurrent work, awaiting APIs, or sharing state.

## Calling and awaiting

An async function explicitly returns System.Threading.Tasks.Task or Task<T>.
Its body returns the inner T, or return;/fallthrough for non-generic Task.
Anonymous async functions follow the same rule. Async procedures/void are not
supported. A synchronous function returning Task returns that object normally
and cannot await merely because its return type is Task.

Calling async code runs synchronously until its first incomplete await; it does
not automatically start a thread. Await suspends rather than blocking that thread.
Task<T>/ValueTask<T> yields T; Task/ValueTask await has no result. Supported
ConfigureAwait forms preserve ordinary .NET continuation-context behavior.
Use ConfigureAwait(false) explicitly when appropriate. Custom awaiters and async
iterators are outside the baseline. ValueTask use must follow its producer's
lifetime/reuse requirements; ordinary Task can be awaited repeatedly.

An async program permits await in the entry block. Its generated CLR entry bridge
waits for the program task before exiting. Functions do not gain an implicit
blocking bridge. Exceptions and cancellation follow normal .NET task behavior.
Requires/ensures run inside the state machine; ensures binds inner T only after
successful cleanup. Failed checks fault the task; fault/cancellation skips ensures.

## Ownership and shared state

Forward CancellationToken explicitly to APIs that accept it. Await does not
inject cancellation or force an operation to honor it. Use CancellationTokenSource
for request/timeout policy. Use Task.Run for suitable CPU work, WhenAll/WhenAny
for coordination, and System.Threading.Channels for channels.

Explicitly await work whose completion belongs to your operation. Discarding or
returning from a function does not automatically join/cancel child tasks or
observe abandoned exceptions. Captured var cells and globals have CLR shared
state behavior. Use normal synchronization or immutable data. Async methods
cannot have ref/out/in parameters; live managed references cannot cross await.

Using const schedules IDisposable cleanup in an async body too. Await using
const selects IAsyncDisposable and awaits each disposal before completing a
scope exit. Sync/async resources share reverse acquisition order. Return, loop
exit, failure, and cancellation all preserve applicable cleanup. Await is allowed
in supported catch/finally bodies, but not exception filters.

```pascal
async program AgentAsync;

uses System as Sys, System.Console as Console, System.IO as IO,
  System.Threading as Threading, System.Threading.Tasks as Tasks;

async function DoubleAfter(Value: int; Cancellation: Threading.CancellationToken): Tasks.Task<int>;
requires Value >= 0;
requires Value <= int.MaxValue div 2;
ensures (Answer: int): Answer = Value * 2;
begin
  await Tasks.Task.Delay(1, Cancellation);
  return Value * 2;
end function;

begin
  using const Cancellation: Threading.CancellationTokenSource :=
    new Threading.CancellationTokenSource();
  const Work: Tasks.Task<int> := DoubleAfter(21, Cancellation.Token);
  const Other: Tasks.Task<int> := Tasks.Task.Run<int>(function(): int => 7);
  const Pending: array of Tasks.Task<int> := [Work, Other];
  const Results: array of int := await Tasks.Task.WhenAll<int>(Pending).ConfigureAwait(false);
  begin
    await using const Stream: IO.MemoryStream := new IO.MemoryStream();
    Stream.WriteByte(65);
    if Stream.Length <> 1L then raise new Sys.InvalidOperationException('Incorrect stream'); end if;
  end;
  if Results[0] <> 42 or Results[1] <> 7 then
    raise new Sys.InvalidOperationException('Incorrect async results');
  end if;
  Console.WriteLine('Async verified');
end;
end program;
```

Read [concurrency](../language/concurrency/README.md) for supported task families and
contexts, [resources](../language/error-handling/async-disposal.md)
for disposal, and [async implementation](../../architecture/async-and-cleanup.md)
when changing state-machine lowering or portable symbols.
