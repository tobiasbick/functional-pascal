# Run a process with bounded text capture

Use System.Diagnostics.Process with a fresh ProcessStartInfo. Set FileName to
the executable and add each argument separately through
[ArgumentList](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.processstartinfo.argumentlist?view=net-10.0).
With UseShellExecute false, arguments are passed directly without shell expansion.
Set WorkingDirectory explicitly and change the child's Environment dictionary;
this leaves the parent's working directory and environment alone.

## Capture and own the child

The following application helper retains at most MaximumCharacters UTF-16 code
units **per stream**, marks truncation, and continues draining discarded output.
Reading stdout and stderr concurrently prevents either pipe from filling while
the parent waits on the other. The limit bounds retained text, not total bytes
produced; the deadline bounds the execution and drain period after process start.
Startup itself is synchronous. This recipe closes stdin immediately.

Save the unit as `ProcessCapture.fpas`. The helpers use actual .NET 10 signatures:
StreamReader.ReadAsync(Memory<char>, CancellationToken) returns ValueTask<int>,
and Process.WaitForExitAsync(CancellationToken) returns Task. Memory<char> is a
normal CLR struct that can cross await; this example does not require Span APIs.

```pascal project=process-capture
// Application-local process execution and text capture.
unit Recipes.ProcessCapture;

uses System as Sys, System.Diagnostics as Diagnostics, System.IO as IO,
  System.Text as Text, System.Threading as Threading,
  System.Threading.Tasks as Tasks;

// A bounded prefix of one output stream.
public type CapturedText = record
  // Retained UTF-16 text.
  public Value: string;
  // True when additional characters were discarded.
  public Truncated: bool;
end record;

// A completed process, including nonzero exit codes.
public type Result = record
  // The child's exit status.
  public ExitCode: int;
  // Standard output, independently bounded.
  public Output: CapturedText;
  // Standard error, independently bounded.
  public Error: CapturedText;
end record;

async function Drain(Reader: IO.StreamReader; MaximumCharacters: int;
  Cancellation: Threading.CancellationToken): Tasks.Task<CapturedText>;
begin
  const Buffer: array of char := new char[4096];
  const Memory: Sys.Memory<char> := new Sys.Memory<char>(Buffer);
  const Prefix: Text.StringBuilder := new Text.StringBuilder();
  var Truncated: bool := false;
  while true do
    const Count: int := await Reader.ReadAsync(Memory, Cancellation).ConfigureAwait(false);
    if Count = 0 then break; end if;
    const Keep: int := Sys.Math.Min(Count, MaximumCharacters - Prefix.Length);
    if Keep > 0 then discard Prefix.Append(Buffer, 0, Keep); end if;
    if Keep < Count then Truncated := true; end if;
  end while;
  return new CapturedText(Prefix.ToString(), Truncated);
end function;

// Runs a fresh start configuration once; callers must not mutate it concurrently.
public async function Run(Options: Diagnostics.ProcessStartInfo;
  TimeoutMilliseconds: int; MaximumCharacters: int;
  Cancellation: Threading.CancellationToken): Tasks.Task<Result>;
requires TimeoutMilliseconds > 0;
requires MaximumCharacters >= 0;
begin
  Cancellation.ThrowIfCancellationRequested();
  Options.UseShellExecute := false;
  Options.CreateNoWindow := true;
  Options.RedirectStandardInput := true;
  Options.RedirectStandardOutput := true;
  Options.RedirectStandardError := true;
  Options.StandardOutputEncoding := Text.Encoding.UTF8;
  Options.StandardErrorEncoding := Text.Encoding.UTF8;
  using const Deadline: Threading.CancellationTokenSource :=
    Threading.CancellationTokenSource.CreateLinkedTokenSource(Cancellation);
  using const Child: Diagnostics.Process := new Diagnostics.Process();
  Child.StartInfo := Options;
  if not Child.Start() then raise new Sys.InvalidOperationException('Process did not start'); end if;
  Deadline.CancelAfter(TimeoutMilliseconds);
  const Output: Tasks.Task<CapturedText> := Drain(Child.StandardOutput, MaximumCharacters, Deadline.Token);
  const Error: Tasks.Task<CapturedText> := Drain(Child.StandardError, MaximumCharacters, Deadline.Token);
  const Readers: array of Tasks.Task<CapturedText> := [Output, Error];
  const Reading: Tasks.Task<array of CapturedText> := Tasks.Task.WhenAll<CapturedText>(Readers);
  try
    Child.StandardInput.Close();
    await Child.WaitForExitAsync(Deadline.Token).ConfigureAwait(false);
    const Captures: array of CapturedText := await Reading.ConfigureAwait(false);
    return new Result(Child.ExitCode, Captures[0], Captures[1]);
  catch Failure: Sys.OperationCanceledException do
    Cancellation.ThrowIfCancellationRequested();
    raise new Sys.TimeoutException('Process execution or output capture timed out', Failure);
  finally
    Deadline.Cancel();
    using const Cleanup: Threading.CancellationTokenSource := new Threading.CancellationTokenSource(5000);
    try
      if not Child.HasExited then
        try
          Child.Kill(true);
        catch Failure: Sys.InvalidOperationException when Child.HasExited do
          // The child exited between the check and Kill.
        end try;
      end if;
      await Child.WaitForExitAsync(Cleanup.Token).ConfigureAwait(false);
    finally
      try
        discard await Reading.WaitAsync(Cleanup.Token).ConfigureAwait(false);
      catch Failure: Sys.OperationCanceledException when Reading.IsCanceled do
        // Both readers have observed cancellation and finished.
      end try;
    end try;
  end try;
end function;

end unit;
```

A nonzero exit code is returned as data. Start failures and I/O failures throw
normal CLR exceptions. A caller cancellation produces OperationCanceledException
with the caller's token; expiry of the execution deadline produces TimeoutException.
The deadline remains active until both streams reach EOF, even if the child has
already exited. Text decoding uses UTF-8 and the usual replacement fallback.
The limit counts UTF-16 code units, so a truncated prefix can end between a
surrogate pair; choose a byte or Unicode-scalar policy if your application needs it.

Cancellation of
[WaitForExitAsync](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.process.waitforexitasync?view=net-10.0)
cancels waiting; it does not terminate a child. The finally block requests
[Kill(true)](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.process.kill?view=net-10.0)
while the direct child is alive, waits for that child, and observes both readers
before disposing the Process. Cleanup has its own five-second bound and can
fail with a CLR exception. Disposing Process alone does not stop a running child.
Kill's tree traversal and WaitForExit apply their documented platform limits:
waiting for the direct child does not prove every descendant has exited, and an
already exited parent cannot be used to kill orphaned descendants. This recipe
is for owned command-line tools, rather than process containment or interactive I/O.

## Call the helper

Place the unit and this program in the same executable project. The
[SDK walkthrough](../../build/standalone-sdk.md) explains creating the project
and selecting a versioned local SDK package. Here `dotnet` is resolved through
PATH; use an absolute executable path when selecting a specific installation.

```pascal project=process-capture
async program CaptureVersion;

uses System as Sys, System.Console as Console, System.Diagnostics as Diagnostics,
  System.Threading as Threading, Recipes.ProcessCapture as Capture;

begin
  const Options: Diagnostics.ProcessStartInfo := new Diagnostics.ProcessStartInfo('dotnet');
  Options.ArgumentList.Add('--version');
  Options.WorkingDirectory := Sys.Environment.CurrentDirectory;
  Options.Environment['DOTNET_NOLOGO'] := '1';
  const Finished: Capture.Result := await Capture.Run(Options, 10000, 128,
    Threading.CancellationToken.None);
  Console.WriteLine(Finished.ExitCode);
  Console.WriteLine(Finished.Output.Value.Trim());
end;
end program;
```

This prints exit code `0` and the selected SDK's version. In an application,
pass its request token instead of CancellationToken.None, check ExitCode, and
handle Output.Truncated and Error.Truncated before interpreting retained text.
Do not share a mutable ProcessStartInfo across concurrent runs.

The Markdown runner compiles both complete sources. The linked
[SDK acceptance test](../../../tests/Fpas.IntegrationTests/Build/MsBuildTests.ProcessRecipe.cs)
calls the documented unit from C# in Debug/Release with controlled child modes:
literal arguments, environment/cwd, stdin EOF, nonzero exit, output larger than
pipe buffers, timeout, cancellation after startup, and launch failure. It verifies
the direct child has exited before timeout/cancellation returns.

## See also

- [Library guides](README.md)
- [Cancellation and coordination](../language/concurrency/cancellation-and-coordination.md)
- [Async disposal and cleanup](../language/error-handling/async-disposal.md)
- [StreamReader.ReadAsync reference](https://learn.microsoft.com/en-us/dotnet/api/system.io.streamreader.readasync?view=net-10.0)
