# Synchronize work and observe task outcomes

Async functions can share mutable objects and captured var bindings. Protect
shared updates explicitly, or give one worker exclusive ownership of the state.
Calling an async function does not create a thread; Task.Run schedules work on
the thread pool when that is appropriate for the workload.

## Hold a semaphore across awaited work

SemaphoreSlim(1, 1) supplies one permit. This application-local helper waits
before entering try/finally, so a canceled wait cannot release an unacquired
permit. Once acquired, the permit is released after success, failure, or
cancellation inside Work. The caller owns the semaphore and its lifetime.

```pascal project=serialized-work
// Application-local serialization of asynchronous work.
unit Recipes.Coordination;

uses System as Sys, System.Threading as Threading, System.Threading.Tasks as Tasks;

// Holds the caller's permit until the supplied work has completed.
public async function UnderGate<T>(
  Gate: Threading.SemaphoreSlim;
  Work: Sys.Func<Threading.CancellationToken, Tasks.Task<T>>;
  Cancellation: Threading.CancellationToken
): Tasks.Task<T>;
begin
  await Gate.WaitAsync(Cancellation).ConfigureAwait(false);
  try
    return await Work(Cancellation).ConfigureAwait(false);
  finally
    discard Gate.Release();
  end try;
end function;
end unit;
```

The selected [WaitAsync(CancellationToken)](https://learn.microsoft.com/en-us/dotnet/api/system.threading.semaphoreslim.waitasync?view=net-10.0)
returns Task. Its timed overloads return Task<bool> and require a separate check
for acquisition. Release returns the previous permit count. SemaphoreSlim is
not recursive and does not establish thread ownership; every access to the
protected state must follow the same discipline.

Place the unit and this program in one project. A completion source releases
three pending jobs together; each shared update then uses the same semaphore.

```pascal project=serialized-work
async program SerializedJobs;

uses
  System as Sys, System.Console as Console, System.Threading as Threading,
  System.Threading.Tasks as Tasks, Recipes.Coordination as Coordination;

begin
  using const Gate: Threading.SemaphoreSlim := new Threading.SemaphoreSlim(1, 1);
  const Start: Tasks.TaskCompletionSource<bool> := new Tasks.TaskCompletionSource<bool>(
    Tasks.TaskCreationOptions.RunContinuationsAsynchronously
  );
  var Total: int := 0;
  async function Add(Value: int): Tasks.Task<int>;
  begin
    discard await Start.Task.ConfigureAwait(false);
    return await Coordination.UnderGate<int>(
      Gate,
      async function(Cancellation: Threading.CancellationToken): Tasks.Task<int> begin
        Cancellation.ThrowIfCancellationRequested();
        Total := Total + Value;
        return Total;
      end function, Threading.CancellationToken.None
    ).ConfigureAwait(false);
  end function;
  const Jobs: array of Tasks.Task<int> := [Add(1), Add(2), Add(3)];
  Start.SetResult(true);
  discard await Tasks.Task.WhenAll<int>(Jobs).ConfigureAwait(false);
  if Total <> 6 then
    raise new Sys.InvalidOperationException('Shared update was lost');
  end if;

  Console.WriteLine(Total);
end;
end program;
```

The program prints `6` and joins all jobs before disposing Gate. Result ordering
from WhenAll follows the input task order; semaphore acquisition order is not a
fairness guarantee. Awaiting while holding a permit can serialize lengthy work;
keep the protected operation as small as the application requires.

## Observe every task, including failures

[WhenAll](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.task.whenall?view=net-10.0)
completes after all supplied tasks finish. Await throws one failure; the joined
task's AggregateException exposes the collected failures. A fault takes priority
over cancellation; a joined task with cancellation and no faults is canceled.
WhenAny returns a completed task to await, and leaves the other tasks running.
Neither operation automatically cancels peers.

This controlled example distinguishes the first completion from the final join,
then observes a separate canceled batch. Completion sources stand in for owned
asynchronous operations; no sleeps or timing assumptions are needed.

```pascal
async program TaskOutcomes;

uses
  System as Sys, System.Console as Console, System.Threading as Threading,
  System.Threading.Tasks as Tasks;

begin
  const First: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>(
    Tasks.TaskCreationOptions.RunContinuationsAsynchronously
  );
  const Second: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>(
    Tasks.TaskCreationOptions.RunContinuationsAsynchronously
  );
  const Jobs: array of Tasks.Task<int> := [First.Task, Second.Task];
  const Joined: Tasks.Task<array of int> := Tasks.Task.WhenAll<int>(Jobs);
  First.SetException(new Sys.InvalidOperationException('First operation failed'));
  const Finished: Tasks.Task<int> := await Tasks.Task.WhenAny<int>(Jobs);
  try
    discard await Finished;
  catch Failure: Sys.InvalidOperationException do
    if Joined.IsCompleted then
      raise new Sys.InvalidOperationException('Other work was not joined');
    end if;

    Console.WriteLine('First failure observed');
  end try;

  Second.SetException(new Sys.ArgumentException('Second operation failed'));
  try
    discard await Joined;
  catch do
    const Failures: Sys.AggregateException? := Joined.Exception;
    if Failures = null or Failures.InnerExceptions.Count <> 2 then
      raise new Sys.InvalidOperationException('Missing task failures');
    end if;

    Console.WriteLine(Failures.InnerExceptions.Count);
  end try;

  using const Cancellation: Threading.CancellationTokenSource := new Threading.CancellationTokenSource();
  Cancellation.Cancel();
  const CanceledJobs: array of Tasks.Task<int> := [
    Tasks.Task.FromResult<int>(1),
    Tasks.Task.FromCanceled<int>(Cancellation.Token)
  ];
  const CanceledJoin: Tasks.Task<array of int> := Tasks.Task.WhenAll<int>(CanceledJobs);
  try
    discard await CanceledJoin;
  catch Failure: Sys.OperationCanceledException when CanceledJoin.IsCanceled do
    Console.WriteLine('Canceled batch observed');
  end try;
end;
end program;
```

The output is `First failure observed`, `2`, and `Canceled batch observed`.
For work that should stop after a peer fails, give it a shared cancellation
source, cancel it at that failure boundary, and still join every owned task.
The [channel pipeline](channels.md) demonstrates that policy. Keep resources
alive until their workers finish, and observe errors from canceled losing work.

The Markdown runner executes both programs. The linked
[SDK acceptance](../../../tests/Fpas.IntegrationTests/Build/MsBuildTests.CoordinationRecipes.cs)
also calls UnderGate from C# in Debug/Release, checking contention, cancellation
before acquisition, work faults, and cancellation while holding the permit.

## See also

- [Library guides](README.md)
- [Channels and producer/consumer ownership](channels.md)
- [Async language rules](../language/concurrency/README.md)
