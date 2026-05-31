# `Std.Proc`

Blocking host process execution for FPAS programs. This page is the full API for the unit.

```pascal
program Example;
uses Std.Console, Std.Proc, Std.Result;

begin
  var Status: Result of integer, string := Std.Proc.Run('fpas', ['--version']);
  if Std.Result.IsOk(Status) then
    WriteLn('exit code: ', Std.Result.Unwrap(Status))
end.
```

`Std.Proc` starts a host process, waits for it to finish, and returns the process exit status. The initial API is intentionally small: it does not expose process handles, stdin/stdout/stderr pipes, environment overrides, or working-directory controls.

**Maintenance (implementers only):** align with [`std_registry/`](../../../crates/fpas-sema/src/std_registry/mod.rs), [`std_calls/proc.rs`](../../../crates/fpas-compiler/src/compiler/std_calls/proc.rs), [`proc.rs`](../../../crates/fpas-std/src/proc.rs), and [`intrinsic/proc.rs`](../../../crates/fpas-bytecode/src/intrinsic/proc.rs).

---

## Importing and names

After `uses Std.Proc;` use **`Run`** or the fully qualified form **`Std.Proc.Run`**.

---

## Quick reference

Requires `uses Std.Proc;`.

| Kind | Name | Notes |
|------|------|-------|
| function | `Run(Command: string; Args: array of string): Result of integer, string` | starts a process, waits for completion, and returns the exit code |

Fallible operations return `Error(message)` with a host error string instead of raising a runtime panic.

---

## Blocking and concurrency

`Run` blocks the thread that executes it until the child process exits. When a call runs inside `go`, it blocks that worker thread only. Combine `go Run(...)` with `Std.Task.Wait` for task-based process workflows.

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

## Implementation map (contributors)

| Concern | Location |
|---------|----------|
| Runtime execution | [`proc.rs`](../../../crates/fpas-std/src/proc.rs) |
| Call lowering | [`std_calls/proc.rs`](../../../crates/fpas-compiler/src/compiler/std_calls/proc.rs) |
| Registration | [`std_registry/loaded/proc.rs`](../../../crates/fpas-sema/src/std_registry/loaded/proc.rs) |
| Intrinsic ids | [`intrinsic/proc.rs`](../../../crates/fpas-bytecode/src/intrinsic/proc.rs) |