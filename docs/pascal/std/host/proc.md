# `Std.Proc`

Blocking host process execution for FPAS programs. This page is the full API for the unit.

```pascal
program Example;
uses Std.Console, Std.Proc;

begin
  case Std.Proc.RunCapture('fpas', ['--version']) of
    Ok(Output):
    begin
      WriteLn(Output.Stdout);
      WriteLn('exit code: ', Output.ExitCode)
    end;
    Error(Message): WriteLn(Message)
  end
end.
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

After `uses Std.Proc;`, public names can be used unqualified or with the
`Std.Proc.` prefix.

---

## Quick reference

Requires `uses Std.Proc;`.

| Kind | Name | Notes |
|------|------|-------|
| record | `ProcessOutput` | captured `ExitCode`, `Stdout`, and `Stderr` |
| function | `CurrentExecutable(): Result of string, string` | returns the absolute path of the running FPAS host executable |
| function | `Run(Command: string; Args: array of string): Result of integer, string` | starts a process, waits for completion, and returns the exit code |
| function | `RunCapture(Command: string; Args: array of string): Result of ProcessOutput, string` | starts a process and captures its exit code and output |

Fallible operations return `Error(message)` with a host error string instead of raising a runtime panic.

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

## `function CurrentExecutable(): Result of string, string`

Returns the absolute path of the executable hosting the running FPAS program.
This allows a tool launched by `fpas` to invoke that same compiler binary.

Returns `Error(message)` if the host cannot determine its executable path.

---

## Blocking and concurrency

`Run` and `RunCapture` block the thread that executes them until the child
process exits. When a call runs inside `go`, it blocks that worker thread only.
Combine a process call in `go` with `Std.Task.Wait` for task-based workflows.

---

## `function Run(Command: string; Args: array of string): Result of integer, string`

Starts `Command` with `Args`, waits for the process to exit, and returns `Ok(exitCode)`.

```pascal
var Status: Result of integer, string := Run('fpas', ['--version']);
if Std.Result.IsError(Status) then
  WriteLn(Std.Result.UnwrapOr(Status, -1))
```

If the process cannot be started, returns `Error(message)`. If the host reports that the process ended without an exit code, returns `Error('process terminated without an exit code')`.

---

## `function RunCapture(Command: string; Args: array of string): Result of ProcessOutput, string`

Starts `Command` with `Args`, waits for it to finish, and returns
`Ok(ProcessOutput)` without writing the child's stdout or stderr to the parent
terminal.

```pascal
case RunCapture('fpas', ['check', 'main.fpas']) of
  Ok(Output):
  begin
    WriteLn(Output.Stdout);
    WriteLn(Output.Stderr)
  end;
  Error(Message): WriteLn(Message)
end
```

A non-zero exit code is a completed process and therefore remains `Ok`; inspect
`Output.ExitCode` to distinguish command success from command failure. A spawn
failure or a process termination without an exit code returns `Error(message)`.

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Runtime execution | [`proc.rs`](../../../../crates/fpas-std/src/proc.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/loaded/proc.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/proc.rs) |
| Intrinsic ids | [`intrinsic/proc.rs`](../../../../crates/fpas-bytecode/src/intrinsic/proc.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)
