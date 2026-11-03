# Bounded channels and producer/consumer ownership

System.Threading.Channels is included in the net10.0 framework references.
A bounded channel with FullMode Wait suspends a writer when its queue is full.
Capacity limits queued items, not the size of each item or the input collection.
Use immutable messages or an explicit ownership rule for mutable payloads.

## Backpressure and completion

This complete example fills a one-item queue, keeps the next WriteAsync pending,
then reads to release capacity. Each ValueTask is awaited once. The writer closes
after its final write; the reader drains the queue before awaiting Completion.

```pascal
async program ChannelBackpressure;

uses System as Sys, System.Console as Console, System.Threading as Threading,
  System.Threading.Tasks as Tasks, System.Threading.Channels as Channels;

begin
  const Pipe: Channels.Channel<int> := Channels.Channel.CreateBounded<int>(1);
  await Pipe.Writer.WriteAsync(10, Threading.CancellationToken.None);
  const Pending: Tasks.ValueTask := Pipe.Writer.WriteAsync(20, Threading.CancellationToken.None);
  if Pending.IsCompleted then raise new Sys.InvalidOperationException('Full queue accepted another item'); end if;
  Console.WriteLine(await Pipe.Reader.ReadAsync(Threading.CancellationToken.None));
  await Pending;
  if not Pipe.Writer.TryComplete() then raise new Sys.InvalidOperationException('Writer was already closed'); end if;
  Console.WriteLine(await Pipe.Reader.ReadAsync(Threading.CancellationToken.None));
  await Pipe.Reader.Completion;
end;
end program;
```

The output is `10` and `20`. TryComplete closes future writes; it does not consume
queued items. Reader.Completion waits for the reader side to finish, so awaiting
it before draining a populated queue can block. Other full modes can drop data;
the [channel reference](https://learn.microsoft.com/en-us/dotnet/core/extensions/channels)
describes their policies.

## Join a pipeline on success, fault, and cancellation

Save this application-local unit as `ChannelPipeline.fpas`. One producer owns
writer completion. One consumer owns the sum, avoiding shared aggregate updates.
A consumer failure cancels a producer waiting for space. A producer failure
completes the writer with that exception, waking the reader. The owner joins both
tasks before its linked cancellation source is disposed.

```pascal project=channel-pipeline
// Application-local bounded aggregation of nonnegative inputs.
unit Recipes.ChannelPipeline;

uses System as Sys, System.Collections.Generic as Generic, System.IO as IO,
  System.Threading as Threading, System.Threading.Tasks as Tasks,
  System.Threading.Channels as Channels;

// Joins one producer and one consumer, including their failure paths.
public async function Sum(Values: Generic.IEnumerable<int>; Capacity: int;
  Cancellation: Threading.CancellationToken): Tasks.Task<int>;
requires Capacity > 0;
begin
  Cancellation.ThrowIfCancellationRequested();
  using const Stop: Threading.CancellationTokenSource :=
    Threading.CancellationTokenSource.CreateLinkedTokenSource(Cancellation);
  const Options: Channels.BoundedChannelOptions := new Channels.BoundedChannelOptions(Capacity);
  Options.FullMode := Channels.BoundedChannelFullMode.Wait;
  Options.SingleWriter := true;
  Options.SingleReader := true;
  Options.AllowSynchronousContinuations := false;
  const Pipe: Channels.Channel<int> := Channels.Channel.CreateBounded<int>(Options);

  async function Produce(): Tasks.Task;
  begin
    try
      for Value: int in Values do
        await Pipe.Writer.WriteAsync(Value, Stop.Token).ConfigureAwait(false);
      end for;
    catch Failure: Sys.Exception do
      discard Pipe.Writer.TryComplete(Failure);
      raise;
    finally
      discard Pipe.Writer.TryComplete();
    end try;
  end function;

  async function Consume(): Tasks.Task<int>;
  begin
    try
      var Total: int := 0;
      while await Pipe.Reader.WaitToReadAsync(Stop.Token).ConfigureAwait(false) do
        var Value: int := 0;
        while Pipe.Reader.TryRead(out Value) do
          Stop.Token.ThrowIfCancellationRequested();
          if Value < 0 then raise new IO.InvalidDataException('Negative pipeline input'); end if;
          Total := Total + Value;
        end while;
      end while;
      return Total;
    catch do
      Stop.Cancel();
      raise;
    end try;
  end function;

  const Sending: Tasks.Task := Produce();
  const Receiving: Tasks.Task<int> := Consume();
  const Workers: Tasks.Task[] := [Sending, Receiving];
  await Tasks.Task.WhenAll(Workers).ConfigureAwait(false);
  return await Receiving.ConfigureAwait(false);
end function;

end unit;
```

WriteAsync returns ValueTask; WaitToReadAsync returns ValueTask<bool>; TryRead
uses an out argument. The
[WaitToReadAsync contract](https://learn.microsoft.com/en-us/dotnet/api/system.threading.channels.channelreader-1.waittoreadasync?view=net-10.0)
returns false on successful completion with no remaining data and throws on a
faulted completion. This loop uses supported ordinary APIs; it needs no async
iterator or await-foreach syntax.

SingleWriter/SingleReader describe this pipeline's actual usage. Keep those
promises accurate if changing the worker count. A channel synchronizes transfer,
not arbitrary mutations of an object after sending it.

Cancellation is cooperative. Synchronous enumeration must itself remain
responsive; a token cannot interrupt a blocking MoveNext. The producer's for-in
scope disposes its enumerator on every exit. On cancellation or consumer failure,
buffered items can remain unconsumed and are abandoned with this private channel;
the failure path joins workers without waiting for Reader.Completion to drain them.
Negative input throws InvalidDataException; checked sum overflow throws
OverflowException. Task fault/cancellation precedence follows WhenAll.

Place the unit and this program in one executable project:

```pascal project=channel-pipeline
async program ChannelSum;

uses System.Console as Console, System.Threading as Threading,
  Recipes.ChannelPipeline as Pipeline;

begin
  const Values: int[] := [10, 20, 30];
  Console.WriteLine(await Pipeline.Sum(Values, 1, Threading.CancellationToken.None));
end;
end program;
```

The program prints `60`. Its complete sources and ChannelBackpressure are checked
by the Markdown runner. The linked
[SDK acceptance](../../../tests/Fpas.IntegrationTests/MsBuildTests.CoordinationRecipes.cs)
calls the exact unit in Debug/Release with empty and larger inputs, producer
enumeration failure, consumer rejection/overflow, and active cancellation.
Enumerator cleanup is checked before the pipeline returns or throws.

## See also

- [Library guides](README.md)
- [Synchronization and task outcomes](task-coordination.md)
- [Cancellation rules](../language/concurrency/cancellation-and-coordination.md)
