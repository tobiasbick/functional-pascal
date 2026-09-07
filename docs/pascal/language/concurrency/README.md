# Concurrency

Go-inspired lightweight task concurrency. Tasks created with `go` may run on worker threads in parallel with the main program; the main program always runs on the OS thread that starts execution.

| Topic | Description |
|-------|-------------|
| [`go`](go.md) | Expression and statement forms, valid call targets |
| [Task handles](task-handles.md) | `task` type, `Wait`, `WaitAll` |
| [Fork-join](fork-join.md) | Parallel work and collecting results |
| [Scheduling](scheduling.md) | Thread pool, cooperative preemption, shared runtime |
| [Channel types](../types/channels.md) | Typed bounded FIFO communication, cancellation, closure |

Per-symbol API: [`Std.Task`](../../std/concurrency/task.md), including cancellation, typed channel
operations, mixed-source `Select`, task groups, supervised retries, and cooperative timed close.

## Bytecode mapping

The compiler lowers `go` to dedicated VM opcodes:

- **`go` as an expression** (e.g. assigned to a `task` variable) emits a **retained** spawn and returns a task handle for later `Wait`.
- **`go` as a statement** (fire-and-forget) emits a **detached** spawn without retaining a handle for the caller.

At startup, the runtime checks verified function metadata for task-start operations. Both retained
and detached `go` spawns, and the `Std.Task.StartTaskInGroup` and `StartSupervisedTask` intrinsics,
mark a function as able to start tasks. Without such metadata, the runtime does not start background
worker threads. `Yield`, channel creation, and group creation alone do not require a pool.

The lifetime of spawned work is bounded by the main task. Tasks that are already runnable are drained during normal teardown; tasks still suspended in `Std.Time.Sleep` are canceled. Use retained spawn plus `Wait` when the main task must observe completion or a return value. See [Scheduling](scheduling.md) for the exact teardown policy.

Explicit groups retain ownership of registered children independently of retained task handles.
Close required groups before leaving the main task. A timed close that reports timeout has not
joined its remaining workers; neither that timeout nor main-task teardown guarantees a hard
wall-clock process exit.

## Keywords

`go`, `channel` — case-insensitive.

## See also

- [Language overview](../README.md)
- [Std.Task API](../../std/concurrency/task.md)
