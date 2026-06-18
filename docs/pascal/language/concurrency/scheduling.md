# Scheduling

## Thread pool

If the compiled program contains **no** spawn opcodes for tasks (equivalently: the program never uses `go` in a way that reaches bytecode), the runtime does **not** start background worker threads.

If it does emit spawn bytecode, the runtime starts **`max(1, available_parallelism − 1)`** worker threads that share a ready queue, while the **main task** (task id `0`) still runs on the thread that started execution. Each pool thread runs **at most one** ready task at a time: workers block when the queue is empty and are woken when work is enqueued or the runtime shuts down. Together, this matches typical machine parallelism without starting idle workers for programs that never spawn tasks.

Background workers exist only for a single program run: the runtime **joins** pool threads before execution returns so short-lived hosts do not accumulate stray threads across many runs.

When the **main task** finishes (normally or with a runtime error), the runtime signals **teardown shutdown** so idle workers wake and exit once the ready queue is drained. This is separate from **task failure**: when a spawned task aborts with a runtime error, other spawned work may be stopped cooperatively at the next instruction boundary. The host surfaces **one** primary diagnostic: if the main task failed, that error wins; otherwise a worker error (for example after a spawned task **`panic`s**) is reported.

## Cooperative scheduling

Spawned tasks can be **preempted cooperatively** after a fixed instruction budget and on the **`Yield`** opcode so long-running bytecode cannot starve other queued tasks on the same worker. The **main** program task always runs on the thread that started execution and is **not** placed on the shared ready queue; a main-thread `Yield` yields the OS thread so pool workers can run.

## Shared runtime state

Worker threads and the main execution thread share one runtime state: immutable bytecode, a mutex-protected **ready queue** of suspended tasks paired with a **condition variable** so idle workers block instead of spinning, **task id** allocation, **task result** storage for handles used with `Wait`, a **shutdown** flag, and mutex-protected **console**, **input**, and **TUI** state so concurrent tasks do not corrupt I/O. Hosted TUI `On*` handlers run on the **main** thread only.

## See also

- [Concurrency overview](README.md) — bytecode mapping for retained vs detached spawn
- [`go`](go.md)
- [Task handles](task-handles.md)
