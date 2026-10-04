# `Std.Proc`

Blocking host process execution for FPAS programs. This page is the full API for the unit.

```pascal
program Example;

uses Std.Console as Console;
uses Std.Proc as Proc;

begin
  case Proc.RunCapture('fpas', ['--version']) of
    when Result.Ok(const Output):
      begin
        Console.WriteLn(Output.Stdout);
        Console.WriteLn('exit code: ', Output.ExitCode);
      end;
    when Result.Error(const Message):
      Console.WriteLn(Message);
  end case;
end program;
```

`Std.Proc` starts a host process and waits for it to finish. A call can either
inherit the parent's output streams or capture stdout and stderr. The unit does
not expose process handles, stdin, environment overrides, or working-directory
controls.

**Trust boundary:** `Run` and `RunCapture` execute arbitrary host commands with
the same privileges as the FPAS process. The runtime does not sandbox or
validate commands beyond starting the requested executable with the supplied
arguments.


## Importing and names

Import with `uses Std.Proc as Proc;`. Access every exported member through `Proc`, for example `Proc.RunCapture(...)`. Imports open no short names.

---

## Quick reference

Requires `uses Std.Proc as Proc;`.

| Kind | Name | Notes |
|------|------|-------|
| record | `ProcessOutput` | captured `ExitCode`, `Stdout`, and `Stderr` |
| function | `CurrentExecutable(): Result of (string, string)` | returns the absolute path of the running FPAS host executable |
| function | `Run(Command: string; Args: array of (string)): Result of (integer, string)` | starts a process, waits for completion, and returns the exit code |
| function | `RunCapture(Command: string; Args: array of (string)): Result of (ProcessOutput, string)` | starts a process and captures its exit code and output |

Fallible operations return `Result.Error(message)` with a host error string instead of raising a runtime panic.

---

## `ProcessOutput`

| Field | Type | Meaning |
|------|------|---------|
| `ExitCode` | `integer` | host process exit code, including non-zero codes |
| `Stdout` | `string` | complete captured standard output |
| `Stderr` | `string` | complete captured standard error |

Captured byte streams are decoded as UTF-8. Invalid byte sequences are replaced
with the Unicode replacement character so process completion remains observable.

---

## `function CurrentExecutable(): Result of (string, string)`

Returns the absolute path of the executable hosting the running FPAS program.
This allows a tool launched by `fpas` to invoke that same compiler binary.

Returns `Result.Error(message)` if the host cannot determine its executable path.

---

## Blocking and concurrency

`Run` and `RunCapture` block the thread that executes them until the child
process exits. When a call runs inside `go`, it blocks that worker thread only.
Combine a process call in `go` with `Std.Tasks.Wait` for task-based workflows.

---

## `function Run(Command: string; Args: array of (string)): Result of (integer, string)`

Starts `Command` with `Args`, waits for the process to exit, and returns `Result.Ok(exitCode)`.

```pascal
uses Std.Console as Console;
uses Std.Proc as Proc;
uses Std.Results as Results;

var Status: result of (integer, string) := Proc.Run('fpas', ['--version']);
if Results.IsError(Status) then
  Console.WriteLn(Results.UnwrapOr(Status, -1));
end if;
```

If the process cannot be started, returns `Result.Error(message)`. If the host reports that the process ended without an exit code, returns `Result.Error('process terminated without an exit code')`.

The child inherits standard input and output. Its standard error is inherited too, except under
`fpas run --diagnostics json` or a native application with `FPAS_DIAGNOSTICS=json`, where each
stderr line becomes a JSON program-output record; see
[machine-readable diagnostics](../../program-structure/cli.md#machine-readable-diagnostics).

---

## `function RunCapture(Command: string; Args: array of (string)): Result of (ProcessOutput, string)`

Starts `Command` with `Args`, waits for it to finish, and returns
`Result.Ok(ProcessOutput)` without writing the child's stdout or stderr to the parent
terminal.

```pascal
uses Std.Console as Console;
uses Std.Proc as Proc;

case Proc.RunCapture('fpas', ['check', 'main.fpas']) of
  when Result.Ok(const Output):
    begin
      Console.WriteLn(Output.Stdout);
      Console.WriteLn(Output.Stderr);
    end;
  when Result.Error(const Message):
    Console.WriteLn(Message);
end case;
```

A non-zero exit code is a completed process and therefore remains `Result.Ok`; inspect
`Output.ExitCode` to distinguish command success from command failure. A spawn
failure or a process termination without an exit code returns `Result.Error(message)`.

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Runtime execution | [`proc.rs`](../../../../crates/fpas-std/src/proc.rs) |
| Hosted `Run` and its stderr receiver | [`hosted/proc.rs`](../../../../crates/fpas-vm/src/vm/hosted/proc.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/loaded/proc.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/proc.rs) |
| Intrinsic ids | [`intrinsic/proc.rs`](../../../../crates/fpas-bytecode/src/intrinsic/proc.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)
