# Cancellation and coordination

Cancellation is explicit through `System.Threading.CancellationToken` and
`CancellationTokenSource`. Accept and forward tokens to cancelable operations;
await does not inject a token or make an API cancelable.

Use Task.Run for appropriate CPU-bound work, Task.WhenAll/WhenAny for coordination,
and System.Threading.Channels for bounded or unbounded channels. These are normal
.NET APIs, not syntax or FPAS wrappers. Task.Run overload resolution uses the
ordinary delegate rules and .NET's task-unwrapping overloads.
For complete semaphore, task-outcome, and producer/consumer workflows, read
[task coordination](../../libraries/task-coordination.md) and
[channels](../../libraries/channels.md).

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

## Example

```pascal
async program DownloadPage;

uses System as Sys,
  System.Console as Console,
  System.Net.Http as Http,
  System.Threading as Threading,
  System.Threading.Tasks as Tasks;

async function Download(Address: string;
  Cancellation: Threading.CancellationToken): Tasks.Task<string>;
begin
  using const Client: Http.HttpClient := new Http.HttpClient();
  return await Client.GetStringAsync(Address, Cancellation);
end function;

begin
  using const Cancellation: Threading.CancellationTokenSource :=
    new Threading.CancellationTokenSource(Sys.TimeSpan.FromSeconds(5.0));
  try
    const Text: string := await Download('https://example.com/', Cancellation.Token);
    Console.WriteLine(Text.Length);
  catch Failure: Sys.OperationCanceledException do
    Console.WriteLine(Failure.Message);
  catch Failure: Http.HttpRequestException do
    Console.WriteLine(Failure.Message);
  end try;
end;
end program;
```

This program needs network access for a successful download. Its integration
test uses a local rejecting proxy to exercise the HTTP failure path. Starting Download
does not imply a new thread. Its Task and CancellationToken are real .NET types.
Client is disposed before Download completes; Cancellation remains alive through
the try/catch and is disposed when the program entry body exits.
For reusable clients, per-request messages, streamed body limits, and cancellation
after headers arrive, follow the [HTTP recipe](../../libraries/http.md).

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
