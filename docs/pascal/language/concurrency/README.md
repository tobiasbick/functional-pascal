# Concurrency

Go-inspired lightweight task concurrency. Tasks created with `go` may run on worker threads in parallel with the main program; the main program always runs on the OS thread that starts execution.

| Topic | Description |
|-------|-------------|
| [`go`](go.md) | Expression and statement forms, valid call targets |
| [Task handles](task-handles.md) | `task` type, `Wait`, `WaitAll` |
| [Fork-join](fork-join.md) | Parallel work and collecting results |
| [Scheduling](scheduling.md) | Thread pool, cooperative preemption, shared runtime |
| [Channel types](../types/channels.md) | Typed bounded FIFO communication, cancellation, closure |

Per-symbol API: [`Std.Task`](../../std/concurrency/task.md).

## Bytecode mapping

The compiler lowers `go` to dedicated VM opcodes:

- **`go` as an expression** (e.g. assigned to a `task` variable) emits a **retained** spawn: the callee and arguments are popped and a task handle is pushed for later `Wait`.
- **`go` as a statement** (fire-and-forget) emits a **detached** spawn: same stack effect except **no** handle is retained for the caller.

At startup, the runtime scans the compiled instruction stream: if the program contains **no** retained or detached spawn opcodes, it does **not** start background worker threads. Opcodes used only for cooperative scheduling (for example **`Yield`**) do **not** by themselves imply a pool — only spawn opcodes do.

The lifetime of spawned work is bounded by the main task. Tasks that are already runnable are drained during normal teardown; tasks still suspended in `Std.Time.Sleep` are canceled. Use retained spawn plus `Wait` when the main task must observe completion or a return value. See [Scheduling](scheduling.md) for the exact teardown policy.

## Keywords

`go`, `channel` — case-insensitive.

## See also

- [Language overview](../README.md)
- [Std.Task API](../../std/concurrency/task.md)
