# Concurrency

Go-inspired lightweight task concurrency. Tasks created with `go` may run on worker threads in parallel with the main program; the main program always runs on the OS thread that starts execution.

| Topic | Description |
|-------|-------------|
| [`go`](go.md) | One owned expression form, exact handle types, valid call targets |
| [Task handles](task-handles.md) | `task` type, `Wait`, `WaitAll` |
| [Task scopes](task-scopes.md) | `scope ... end scope`: owned child tasks, cancellation, failures, handle escapes |
| [Fork-join](fork-join.md) | Parallel work and collecting results |
| [Scheduling](scheduling.md) | Thread pool, cooperative preemption, shared runtime |
| [Channel types](../types/channels.md) | Typed bounded FIFO communication, cancellation, closure |

Per-symbol API: [`Std.Tasks`](../../std/concurrency/task.md), including cancellation, typed channel
operations, mixed-source `Select`, task groups, supervised retries, and cooperative timed close.

## Bytecode mapping

The compiler lowers `go` to dedicated VM opcodes:

- Every `go` expression emits a retained spawn: `task of T` for a function returning T, or `task` for a procedure.
- The same-routine [task scope](task-scopes.md) enrolls and retains the child, including for `discard go Work();`. Dedicated opcodes enter a scope, leave it normally, and leave it with cancellation.

At startup, the runtime checks verified function metadata for task-start operations. `go`
spawns and the `Std.Tasks.StartTaskInGroup` and `StartSupervisedTask` intrinsics
mark a function as able to start tasks. Without such metadata, the runtime does not start background
worker threads. `Yield`, channel creation, and group creation alone do not require a pool.

Every task has an owner. Scope children end before their scope does. Groups
belong to their creating task; when it terminates, remaining groups are cancelled
and fully closed before the task's terminal outcome is published. Use `Wait`
to retrieve a required result before the owner releases it. See [scheduling](scheduling.md).

Explicit groups retain ownership of registered children independently of retained task handles.
Close groups explicitly when their results/reports are needed. A timed close
that reports timeout retains unfinished children. Cooperative join, including
main-task cleanup, has no hard wall-clock completion guarantee.

## Keywords

`go`, `channel`, `scope` — case-insensitive.

## See also

- [Language overview](../README.md)
- [Std.Tasks API](../../std/concurrency/task.md)
