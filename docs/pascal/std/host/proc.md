# `Std.Os.Proc`

Host process execution and interactive binary pipes for FPAS programs.

```pascal id=example-1
program Example;

uses Std.Console as Console, Std.Os.Proc as Proc;

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

`Std.Os.Proc` starts a host process and waits for it to finish. A call can either
inherit the parent's output streams or capture stdout and stderr. The options
forms configure child stdin, environment, working directory, timeout,
cancellation, and capture bounds. `Start` returns an owned process handle for
incremental binary I/O.

**Trust boundary:** `Run` and `RunCapture` execute arbitrary host commands with
the same privileges as the FPAS process. The runtime does not sandbox or
restrict the programs that may be launched. Configuration validation runs
before launch.


## Importing and names

After `uses Std.Os.Proc as Proc;`, access functions and records through that alias.

---

## Quick reference

Requires `uses Std.Os.Proc as Proc;`.

| Kind | Name | Notes |
|------|------|-------|
| type | `ProcessOutput` | captured `ExitCode`, `Stdout`, and `Stderr` |
| type | `EnvironmentVariable` | `Name: string` and `Value: Option of string`; `None` removes the child variable |
| type | `ProcessOptions` | child configuration; start from `DefaultOptions()` |
| type | `ProcessExitStatus` | optional native `ExitCode` and terminating `Signal` |
| function | `CurrentExecutable(): Result of (string, Proc.ProcError)` | returns the absolute path of the running FPAS host executable |
| function | `Run(Command: string; Args: array of string): Result of (integer, Proc.ProcError)` | starts a process, waits for completion, and returns the exit code |
| function | `RunCapture(Command: string; Args: array of string): Result of (ProcessOutput, Proc.ProcError)` | captures output with a 1 MiB limit per raw stream and stdin at EOF |
| function | `DefaultOptions(): ProcessOptions` | returns default configuration without launching a process |
| function | `RunWithOptions(Command: string; Args: array of string; Options: ProcessOptions): Result of (integer, Proc.ProcError)` | configures stdin/environment/cwd and inherits output |
| function | `RunCaptureWithOptions(Command: string; Args: array of string; Options: ProcessOptions): Result of (ProcessOutput, Proc.ProcError)` | configures the child and captures bounded output |
| function | `RunInteractive(Command: string; Args: array of string; Options: ProcessStartOptions; TimeoutMillis: Option of integer; CancellationToken: Option of Tasks.CancellationToken): Result of (ProcessExitStatus, Proc.ProcError)` | uses an explicit host terminal with owned timeout/cancellation cleanup |
| function | `TryWaitStatus(Process: Process): Result of (Option of ProcessExitStatus, Proc.ProcError)` | probes exact native completion without consuming the cached status or closing pipes |
| function | `ReadStdoutCase(Process: Process; MaxBytes: integer; Callback: procedure(Outcome: result of (array of integer, Proc.ProcError))): Tasks.WaitCase` | selects one bounded stdout read or EOF |
| function | `ReadStderrCase(Process: Process; MaxBytes: integer; Callback: procedure(Outcome: result of (array of integer, Proc.ProcError))): Tasks.WaitCase` | selects one bounded stderr read or EOF |
| function | `WriteStdinCase(Process: Process; Bytes: array of integer; Callback: procedure(Outcome: result of (integer, Proc.ProcError))): Tasks.WaitCase` | selects one partial stdin write |
| function | `CloseStdinCase(Process: Process; Callback: procedure(Outcome: result of (boolean, Proc.ProcError))): Tasks.WaitCase` | selects flushing and closing stdin |
| function | `WaitStatusCase(Process: Process; Callback: procedure(Outcome: result of (ProcessExitStatus, Proc.ProcError))): Tasks.WaitCase` | selects exact native completion without closing the handle |

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

In pool tasks and the default all-stop debugger, all Run variants, `RunInteractive`,
`Start` and process `Close` suspend their task while an owned native job executes.
Launch, pipe supervision, terminal-provider calls, foreground restoration, stderr
receivers and child reaping therefore leave the VM worker available, including
inside inline FPAS callbacks. Main-program calls and explicit synchronous native
evaluation helpers retain their calling-thread behavior.

Each VM admits at most 64 native process jobs, including results not yet delivered.
Saturation returns `ProcError.LimitExceeded('native process jobs', 64)` before
starting another native thread. `Close` waits for admission instead, so saturation
does not prevent eventual handle cleanup. The existing 64 owned-process and 64
interactive-handle limits remain separate. Inputs own native strings and options;
FPAS values, frames and result construction stay on the VM task.

Timeout budgets begin before copying native inputs and worker admission. The
shared process engine probes each stream in bounded 8 KiB chunks and retries idle
work after one millisecond. The VM polls job completion through its shared timer.
Native supervision and host stderr delivery continue while all-stop FPAS execution
is paused; the saved task consumes the result only after it resumes.

Cancellation signals and joins native work. An OS launch, terminal provider, host
receiver or reaping call already in progress cannot be forcibly interrupted, so
cleanup can wait for that call. Callbacks may request VM shutdown: jobs retired
inside a native process callback or host shutdown request are joined after callbacks
return, avoiding self-joins and callback/provider lock cycles. Requesting shutdown
does not itself wait for process joins; the VM's run/teardown barrier does.
An unpublished `Start` result is closed
on abandonment; published process handles remain the caller's responsibility.

Abandoning a call through task cancellation, scope exit, a root failure or VM shutdown closes
its owned group and reaps the direct child before releasing the saved state. Its pipe state
and input buffer are released as well. Child descendants must remain in their inherited
process group or job. Independently returned `Process` handles require explicit `Close`
or VM teardown; abandoning a selection does not close those handles.
Unconsumed captures and unfinished forwarded stderr lines are discarded on task abandonment.

`Run` delivers stderr lines live on the VM thread when a receiver is installed.
`RunWithOptions` delivers bounded stderr after process and pipe cleanup. Receivers can request
VM shutdown; they do not run under the process-registry lock. Calls that capture output do
not use this receiver. Existing exit-code, partial-output and typed-error contracts apply.

---

## Selectable interactive pipes

The five process case constructors return `Tasks.WaitCase` for
[`Tasks.Select`](../concurrency/task.md#mixed-source-selection). Construction does no
pipe I/O and does not observe child completion. Ordered selection probes commit the
first ready operation; later cases do not read, write, close stdin or observe status.
An earlier ready timer or cancellation case therefore leaves the process untouched.

Each case belongs to its creating task and is single-use. Selection consumes all supplied
cases and invokes only the winner's callback. Closing, losing or abandoning a case releases
its callback and any copied write bytes. It leaves the independently owned process and
its pipes open. Use `Proc.Close` to release the process explicitly. Scope cancellation
releases suspended cases; VM teardown also closes all remaining process handles.

Pool tasks and the all-stop debugger release their worker while no case is ready,
including inside inline callbacks. Native readiness is polled through the existing
selection scheduler: one millisecond for pool/debugger probes, at most ten milliseconds
between synchronous root probes. No native I/O thread is created for a selection.
All-stop pause suspends probing while external children continue running.

Combine process cases with `Tasks.TimerCase` or `Tasks.CancellationCase` for timeout or
cancellation policy. Rebuilding a timer starts a new interval at each `Select` call;
preserve an overall deadline in the calling program when needed. Drain stdout and
stderr while writing large inputs to avoid pipe backpressure deadlocks.

Invalid read bounds, byte values or callback shapes are runtime selection errors
(`RUNTIME_INVALID_TASK` for invalid bounds/bytes). Native failures and handles that are
closed or belong to another VM produce the winning callback's `Error(ProcError)`.
Pipe failures have empty capture fields; the callback receives only that operation's
outcome. A case retains a handle identity rather than extending a closed process's life.

### `ReadStdoutCase` and `ReadStderrCase`

`Process` identifies a process returned by `Start`. `MaxBytes` is in `1..1048576`.
The callback receives `Ok(Bytes)` with at most that many bytes, each in `0..255`.
No currently available bytes leave the case pending; an empty array means EOF and
remains observable by later cases. The two functions differ only in the selected stream.

### `WriteStdinCase`

`Bytes` contains at most 1048576 integers in `0..255`, copied at construction.
The callback receives `Ok(Count)` after a positive partial write. Retry the unwritten
suffix with a new case. A full pipe leaves the case pending. Empty input wins with
`Ok(0)` when stdin is open; closed stdin yields a typed error.

### `CloseStdinCase`

The callback receives `Ok(true)` after flushing and closing stdin. Pending flush leaves
the case pending. Repeated close also returns `Ok(true)`. This sends EOF to the child
without closing its output pipes or terminating the process.

### `WaitStatusCase`

The callback receives `Ok(ProcessExitStatus)` after direct-child completion. Native
exit codes and terminating signals follow `TryWaitStatus`. The cached status remains
observable, and stdout/stderr remain available for draining. Observing completion also
terminates remaining descendants in the owned group, following the process ownership
contract. This case neither closes nor consumes the process handle.

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

uses Std.Os.Proc as Proc, Std.Test as Test;

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
shutdown terminate the owned process tree, reap the child, and finish owned I/O
before returning. Unix uses an owned process group; Windows uses a
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
| `TryWaitStatus(Process: Process)` | `Result of (Option of ProcessExitStatus, ProcError)` | Return `None` while running or the exact cached native completion status. |
| `Terminate(Process: Process)` | `Result of (boolean, ProcError)` | Request termination of the owned process group; retain the pipes and handle. |
| `Close(Process: Process)` | `Result of (boolean, ProcError)` | Terminate, reap, release pipes and invalidate every alias. |

Pipe probes, `TryWait` and `TryWaitStatus` do not wait for data, a newline or child completion.
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

## `ProcessExitStatus` and `TryWaitStatus`

| Field | Type | Meaning |
| --- | --- | --- |
| `ExitCode` | `Option of integer` | Native exit code; absent on Unix signal termination |
| `Signal` | `Option of string` | Unix canonical signal name, such as `SIGTERM`, or decimal signal text when no canonical name is known; absent on Windows |

`TryWaitStatus` reads the same cached native completion as `TryWait`. Both views
remain available after pipe EOF and repeated probes; neither consumes the other.
On Windows the precise view preserves the unsigned 32-bit native exit code in
`integer`, including values above 2147483647. The legacy integer view retains
its signed host-code conversion and `-1` fallback for signal termination.
Observing either status view retains the handle and output pipes until `Close`
and terminates remaining owned descendants according to the process-group rules.

## `RunInteractive`

```pascal id=interactive-login-check
program InteractiveLogin;

uses Std.Os.Proc as Proc;

begin
  const Status := Proc.RunInteractive('agent', ['login'], Proc.DefaultStartOptions(), Some(60000), None);
end.
```

`RunInteractive` uses the host's explicitly supplied interactive input, output
and error streams while applying `ProcessStartOptions`. It never uses ACP
protocol pipes, queued Console input or debugger protocol stdin. Commands and
arguments are passed separately, without shell interpolation. A nonzero exit
or Unix signal termination is `Ok(ProcessExitStatus)`; the application decides
whether that status represents successful authentication.

The CLI and native executable runner offer inherited streams only when stdin,
stdout and stderr are actual terminals. Without terminal support the function
returns `InvalidArgument('Terminal', ...)` before launching a child. Embedded
VMs and debugger hosts explicitly install a native `InteractiveProcessTerminal`
provider using `set_interactive_process_terminal`; VM/debugger constructors do
not grant access to OS streams. Providers can supply independent scripted
streams or a host terminal. A Unix controlling-terminal provider transfers
foreground ownership to the child's process group and restores the previous
foreground group on completion or failure. This does not create a PTY.

Pool/debugger calls invoke the thread-safe provider on their native process job.
An in-flight provider call must return before abandonment can finish joining it.

`None` timeout or cancellation token leaves that bound absent. Negative timeout
and invalid configuration are rejected first; an already-cancelled token
returns `Cancelled`, including with zero timeout. A zero timeout returns
`Timeout` without opening terminal streams or launching a child. During a run,
cancellation precedes a ready status, which precedes timeout. Invalid or foreign
cancellation tokens fail before launch. No process output is captured; error
capture fields remain empty. Timeout, cancellation, VM shutdown and escaping
runtime failure terminate and reap the owned group before releasing ownership.
An enclosing task-scope failure also interrupts the call when no explicit
cancellation token or timeout was supplied; the scope retains its original
failure after child cleanup.
Applications should release any active raw Console terminal mode before handing
the terminal to an interactive child.

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Pure defaults and executable path | [`proc.rs`](../../../../crates/fpas-runtime/src/stdlib/proc.rs) |
| Hosted process supervision | [`hosted/proc/mod.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/proc/mod.rs) |
| Explicit native terminal provider | [`process_terminal.rs`](../../../../crates/fpas-runtime/src/execution/process_terminal.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/loaded/proc.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/proc.rs) |
| Intrinsic ids | [`intrinsic/proc.rs`](../../../../crates/fpas-code/src/intrinsic/proc.rs) |

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
`Std.Os.Proc.RunCaptureWithOptions`. Capture failures retain the bounded output
collected before termination; directly inherited streams have empty captured
fields. Prelaunch failures have no captured output. A successfully completed
child with a nonzero exit code remains an `Ok` outcome.
