# `Std.Proc`

Host process execution and interactive binary pipes for FPAS programs.

```pascal id=example-1
program Example;

uses Std.Console as Console, Std.Proc as Proc;

begin
  case Proc.RunCapture('fpas', ['--version']) of
    when Ok(const Output):
      begin
        Console.WriteLn(Output.Stdout);
        Console.WriteLn('exit code: ', Output.ExitCode);
      end;
    when Error(const Message):
      Console.WriteLn(Message);
  end case;
end.
```

`Std.Proc` starts a host process and waits for it to finish. A call can either
inherit the parent's output streams or capture stdout and stderr. The options
forms configure child stdin, environment, working directory, timeout,
cancellation, and capture bounds. `Start` returns an owned process handle for
incremental binary I/O.

**Trust boundary:** `Run` and `RunCapture` execute arbitrary host commands with
the same privileges as the FPAS process. The runtime does not sandbox or
restrict the programs that may be launched. Configuration validation runs
before launch.


## Importing and names

After `uses Std.Proc as Proc;`, access functions and records through that alias.

---

## Quick reference

Requires `uses Std.Proc as Proc;`.

| Kind | Name | Notes |
|------|------|-------|
| type | `ProcessOutput` | captured `ExitCode`, `Stdout`, and `Stderr` |
| type | `EnvironmentVariable` | `Name: string` and `Value: Option of string`; `None` removes the child variable |
| type | `ProcessOptions` | child configuration; start from `DefaultOptions()` |
| function | `CurrentExecutable(): Result of (string, Proc.ProcError)` | returns the absolute path of the running FPAS host executable |
| function | `Run(Command: string; Args: array of string): Result of (integer, Proc.ProcError)` | starts a process, waits for completion, and returns the exit code |
| function | `RunCapture(Command: string; Args: array of string): Result of (ProcessOutput, Proc.ProcError)` | captures output with a 1 MiB limit per raw stream and stdin at EOF |
| function | `DefaultOptions(): ProcessOptions` | returns default configuration without launching a process |
| function | `RunWithOptions(Command: string; Args: array of string; Options: ProcessOptions): Result of (integer, Proc.ProcError)` | configures stdin/environment/cwd and inherits output |
| function | `RunCaptureWithOptions(Command: string; Args: array of string; Options: ProcessOptions): Result of (ProcessOutput, Proc.ProcError)` | configures the child and captures bounded output |

Fallible operations return `Error(Cause)` with a typed `Proc.ProcError`. Host-backed
variants retain the original native `HostError` cause; operational failures do not
raise a runtime panic.

---

## `ProcessOutput`

| Field | Type | Meaning |
|------|------|---------|
| `ExitCode` | `integer` | host process exit code, including non-zero codes |
| `Stdout` | `string` | complete captured standard output |
| `Stderr` | `string` | complete captured standard error |

Captured byte streams are decoded as UTF-8. Invalid byte sequences are replaced
with the Unicode replacement character so process completion remains observable.
Limits count the original stream bytes before this lossy decoding.

---

## `function CurrentExecutable(): Result of (string, Proc.ProcError)`

Returns the absolute path of the executable hosting the running FPAS program.
This allows a tool launched by `fpas` to invoke that same compiler binary.

Returns `Error(Cause)` if the host cannot determine its executable path.

---

## Blocking and concurrency

All four Run functions block the thread that executes them until the child
process exits. When a call runs inside `go`, it blocks that worker thread only.
Combine a process call in `go` with `Std.Tasks.Wait` for task-based workflows.

---

## `function Run(Command: string; Args: array of string): Result of (integer, Proc.ProcError)`

Starts `Command` with `Args`, waits for the process to exit, and returns `Ok(exitCode)`.

```pascal id=example-2 fragment context=body-console-proc
const Status: result of (integer, Proc.ProcError) := Proc.Run('fpas', ['--version']);
case Status of
  when Ok(_):
    null;
  when Error(_):
    Console.WriteLn(-1);
end case;
```

If the process cannot be started, returns `Error(ProcError.StartFailed(Cause))`.
Wait failures or termination without an exit code return
`WaitFailed(Cause)`. Pipe I/O failures return `IoFailure(Cause)`.

The child inherits standard input and output. Its standard error is inherited too, except under
`fpas run --diagnostics json`, `fpas test --diagnostics json`, or a native application with `FPAS_DIAGNOSTICS=json`, where each
stderr line becomes a JSON program-output record; see
[machine-readable diagnostics](../../program-structure/cli.md#machine-readable-diagnostics).
This live forwarding limits an unfinished stderr line to 1 MiB. Completed
lines are forwarded as they arrive without a total-stream capture bound.

---

## `function RunCapture(Command: string; Args: array of string): Result of (ProcessOutput, Proc.ProcError)`

Starts `Command` with `Args`, waits for it to finish, and returns
`Ok(ProcessOutput)` without writing the child's stdout or stderr to the parent
terminal.

```pascal id=example-3 fragment context=body-console-proc
case Proc.RunCapture('fpas', ['check', 'main.fpas']) of
  when Ok(const Output):
    begin
      Console.WriteLn(Output.Stdout);
      Console.WriteLn(Output.Stderr);
    end;
  when Error(const Message):
    Console.WriteLn(Message);
end case;
```

A non-zero exit code is a completed process and therefore remains `Ok`; inspect
`Output.ExitCode` to distinguish command success from command failure. A spawn
failure or a process termination without an exit code returns `Error(Cause)`.
Stdin is closed immediately. Stdout and stderr are each limited to
**1_048_576 raw bytes**. Exceeding either limit returns `LimitExceeded` with
the bounded captured output, rather than an unbounded `ProcessOutput`.

---

## `ProcessOptions` and `DefaultOptions`

Use `Proc.DefaultOptions()` and a record update to change selected settings.
The native record fields have no declaration defaults; constructing
`Proc.ProcessOptions(...)` directly requires every field.

| Field | Type | Value from DefaultOptions |
|-------|------|---------------------------|
| `WorkingDirectory` | `Option of string` | `None`; inherit the parent's working directory |
| `Environment` | `array of Proc.EnvironmentVariable` | `[]`; inherit the parent environment unchanged |
| `Input` | `string` | `''`; write these UTF-8 bytes to child stdin, then close it |
| `TimeoutMillis` | `Option of integer` | `None`; no process timeout |
| `MaxOutputBytes` | `integer` | `1_048_576`; maximum raw bytes in each captured stream |
| `CancellationToken` | `Option of Tasks.CancellationToken` | `None`; no cancellation token |

An environment entry is constructed with both fields, for example
`Proc.EnvironmentVariable(Name := 'MODE', Value := Some('test'))`.
`Some('')` sets an empty value; `None` removes the variable from the child.
Other variables are inherited. Neither environment overrides nor the child's
working directory change the parent process.

```pascal id=example-options run
program ProcessDefaults;

uses Std.Proc as Proc, Std.Test as Test;

begin
  const Defaults := Proc.DefaultOptions();
  Test.AssertTrue(Defaults.WorkingDirectory is None);
  Test.AssertEquals(0, Defaults.Environment.Length());
  Test.AssertEquals('', Defaults.Input);
  Test.AssertTrue(Defaults.TimeoutMillis is None);
  Test.AssertEquals(1_048_576, Defaults.MaxOutputBytes);
  Test.AssertTrue(Defaults.CancellationToken is None);
  const Options := Defaults with
    Input := 'hello'#10;
    TimeoutMillis := Some(5000);
    Environment := [Proc.EnvironmentVariable(Name := 'MODE', Value := Some('test'))];
  end with;
  Test.AssertEquals('hello'#10, Options.Input);
  Test.AssertEquals('', Defaults.Input);
end.
```

## Calls with options

`RunWithOptions` returns the exit code and inherits stdout. It inherits stderr
when no program-stderr receiver is installed. With the CLI's JSON receiver,
stderr is collected up to MaxOutputBytes and forwarded after child and pipe
cleanup. This deferred forwarding differs from the legacy Run function's
live line forwarding.
`RunCaptureWithOptions` returns `ProcessOutput` and captures both streams.
Both supply `Options.Input` and then EOF, including with default options.
This differs from the original Run function, which inherits stdin. NUL is
allowed in Input as a data byte.

Input writing and stdout/stderr reading proceed concurrently, so a child may
write output before reading its input without filling one pipe and blocking
all progress. `MaxOutputBytes` applies separately to raw stdout and stderr
in the capture form. It may be zero for a command that produces no captured
output. It also bounds stderr collected for a RunWithOptions program-stderr
receiver; directly inherited streams have no capture bound.

`Some(0)` is an immediate timeout. An already cancelled token or an immediate
timeout prevents launch. During execution, timeout, cancellation, and VM
shutdown terminate the owned process tree, reap the child, and join I/O
workers before returning. Unix uses an owned process group; Windows uses a
Job Object for the child and its descendants. Remaining owned descendants are
also terminated when the direct child finishes. Unix descendants must remain
in the inherited process group; deliberately detached processes and passing
pipe handles to unrelated processes are outside this API.
The timeout covers the managed child and its pipes. A synchronous host
program-stderr receiver runs after cleanup and cannot be interrupted by that
deadline. RunCaptureWithOptions uses no receiver.

Capture remains UTF-8 with replacement for invalid byte sequences, and a
normal nonzero child exit remains `Ok`. Use the error's captured fields after
timeout, cancellation, output overflow, or execution failure.

## Configuration validation and limits

Validation finishes before a child is launched. Command, arguments, working
directory, and environment names/values must not contain NUL. Environment
names must also be nonempty and contain no `=`. Duplicate override names
are rejected; comparison is case-insensitive on Windows and case-sensitive
on Unix. TimeoutMillis must be nonnegative when present.

| Resource | Bound |
|----------|-------|
| Arguments (`arguments`) | 4096 |
| Environment override entries (`environment`) | 1024 |
| Combined command, arguments, working directory, and environment names/values (`configuration bytes`) | 1_048_576 UTF-8 bytes |
| Input (`input bytes`) | 67_108_864 UTF-8 bytes, separate from the configuration budget |
| MaxOutputBytes | `0..67_108_864` per captured raw stream |
| Simultaneously owned processes per VM (`processes`) | 64 |

Capture-limit errors identify `stdout bytes` or `stderr bytes` and the
configured MaxOutputBytes. Other resource errors use the labels above.

## Interactive processes

`Process` is an opaque VM-owned handle. Copies alias the same child and pipes.
`ProcessStartOptions` contains `WorkingDirectory: Option of string` and
`Environment: array of EnvironmentVariable`; `DefaultStartOptions()` initializes
them to `None` and `[]`. Launch validation follows the configuration rules above.

| Function | Result | Meaning |
|----------|--------|---------|
| `DefaultStartOptions()` | `ProcessStartOptions` | Construct default launch options without host effects. |
| `Start(Command: string; Args: array of string; Options: ProcessStartOptions)` | `Result of (Process, ProcError)` | Start a child with separate raw stdin, stdout and stderr pipes. |
| `ReadStdout(Process: Process; MaxBytes: integer)` | `Result of (Option of array of integer, ProcError)` | Probe stdout: `None` pending, `Some(Bytes)` data, `Some([])` EOF. |
| `ReadStderr(Process: Process; MaxBytes: integer)` | `Result of (Option of array of integer, ProcError)` | Probe stderr with the same pending/data/EOF distinction. |
| `WriteStdin(Process: Process; Bytes: array of integer)` | `Result of (integer, ProcError)` | Write a possibly partial prefix; zero means retry when a nonempty write is pending. |
| `CloseStdin(Process: Process)` | `Result of (boolean, ProcError)` | Flush accepted writes and send EOF; false means pending, true means closed. |
| `TryWait(Process: Process)` | `Result of (Option of integer, ProcError)` | Return `None` while running or `Some(ExitCode)` after exit. |
| `Terminate(Process: Process)` | `Result of (boolean, ProcError)` | Request termination of the owned process group; retain the pipes and handle. |
| `Close(Process: Process)` | `Result of (boolean, ProcError)` | Terminate, reap, release pipes and invalidate every alias. |

Pipe probes and `TryWait` do not wait for data, a newline or child completion.
Read both output streams regularly, retry pending operations with a short sleep,
and advance writes only by their returned byte count. The caller owns its polling
deadline and cancellation policy; `Terminate` or `Close` stops the child. No
output is captured in an unbounded runtime buffer. Each read accepts
`MaxBytes` in `1..1048576`; each write validates at most 1 MiB of integer bytes
in `0..255` before sending anything. Empty writes return zero on open stdin.
Bytes are never decoded or normalized; a Unicode sequence may span reads.

`CloseStdin` is idempotent. While it returns false, keep draining outputs and
retry it until true. Writing after stdin closure fails. EOF reads are repeatable
and can become available after `TryWait` has already returned an exit code.
Nonzero exits are successful results; signal termination without a native exit
code is represented by `-1`. Remaining owned descendants are terminated when
`TryWait` observes direct-child completion. This shares the process-group rules
of the blocking APIs above.

At most 64 interactive handles may be open per VM, independently of the blocking
process-call limit. Close handles explicitly, including completed children.
VM shutdown, normal completion and escaping runtime failures terminate and reap
all remaining children and release their pipes. A closed or foreign handle
returns `InvalidArgument('Process', ...)`; repeated Close fails. Invalid byte or
read-size arguments use InvalidArgument, oversized writes use LimitExceeded,
and native launch/I/O/wait failures retain their typed host cause. Interactive
calls have no accumulated capture, so capture fields in errors are empty.

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Pure defaults and executable path | [`proc.rs`](../../../../crates/fpas-std/src/proc.rs) |
| Hosted process supervision | [`hosted/proc/mod.rs`](../../../../crates/fpas-vm/src/vm/hosted/proc/mod.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/loaded/proc.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/proc.rs) |
| Intrinsic ids | [`intrinsic/proc.rs`](../../../../crates/fpas-bytecode/src/intrinsic/proc.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)

## Typed failures

`ProcError.StartFailed(Cause)` reports launch failure. Legacy Run and RunCapture
retain `WaitFailed(Cause)` for wait/termination failures and `IoFailure(Cause)`
for pipe I/O failures. CurrentExecutable also uses IoFailure.
Cause is the original [HostError](errors.md). Invalid configuration uses
`InvalidArgument(Name, Reason)`.

The options APIs and bounded captures also return:

| Variant | Payload |
|---------|---------|
| `Timeout` | `Operation: string; Stdout: string; Stderr: string` |
| `Cancelled` | `Operation: string; Stdout: string; Stderr: string` |
| `LimitExceeded` | `Operation: string; Resource: string; Limit: integer; Stdout: string; Stderr: string` |
| `ExecutionFailed` | `Cause: Errors.HostError; Stdout: string; Stderr: string` |

ExecutionFailed reports failures after launch in the options APIs, including
wait, signal/termination, and pipe I/O failures.

Operation names identify the actual function, such as
`Std.Proc.RunCaptureWithOptions`. Capture failures retain the bounded output
collected before termination; directly inherited streams have empty captured
fields. Prelaunch failures have no captured output. A successfully completed
child with a nonzero exit code remains an `Ok` outcome.
