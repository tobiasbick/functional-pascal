# Scheduling

## Thread pool

If the compiled program contains **no** spawn opcodes for tasks (equivalently: the program never uses `go` in a way that reaches bytecode), the runtime does **not** start background worker threads.

If it does emit spawn bytecode, the runtime starts **`max(1, available_parallelism − 1)`** worker threads that share a ready queue, while the **main task** (task id `0`) runs on the thread that started execution. Each pool thread runs **at most one** ready task at a time: workers block when the queue is empty and are woken when work is enqueued or the runtime shuts down. Together, this matches typical machine parallelism without starting idle workers for programs that never spawn tasks.

Background workers exist only for a single program run: the runtime **joins** pool threads before execution returns so short-lived hosts do not accumulate stray threads across many runs.

When the **main task** finishes, the runtime begins **teardown shutdown**. Idle workers wake and exit after draining tasks that were already in the ready queue. Spawned tasks that are still suspended in `Std.Time.Sleep`, and ready tasks that try to sleep after teardown has begun, are canceled instead of delaying program exit. A retained canceled task is completed with a shutdown error; code that needs its result must call `Wait` before the main task finishes. Detached sleeping tasks are canceled without a result because they have no handle.

Teardown is separate from **task failure**: when a spawned task aborts with a runtime error, other spawned work may be stopped cooperatively at the next instruction boundary. The host surfaces **one** primary diagnostic: if the main task failed, that error wins; otherwise a worker error (for example after a spawned task **`panic`s**) is reported.

## Cooperative scheduling

Spawned tasks can be **preempted cooperatively** after a fixed instruction budget and on the **`Yield`** opcode so long-running bytecode cannot starve other queued tasks on the same worker. The shared ready queue is **FIFO**: the oldest suspended task is resumed first. The **main** program task always runs on the thread that started execution and is **not** placed on the shared ready queue; a main-thread `Yield` yields the OS thread so pool workers can run.

`Std.Time.Sleep` is also a cooperative suspension point for spawned tasks. Sleeping tasks are grouped
by millisecond deadline in a shared timer queue. One timer-driver thread moves each due group to the
ready queue, so sleeping tasks do not occupy pool workers. `Sleep` on the main task remains a blocking
host wait.

Synchronous hosted callbacks execute as part of their owner task. If callback bytecode reaches
`Yield` or `Std.Time.Sleep`, the VM saves both the callback frame and the partially completed hosted
operation, releases the pool worker, and resumes the same owner later. Already completed callback
elements are not invoked again, and no separate task identity is created for the callback.

## Shared runtime state

Worker threads and the main execution thread share one runtime state: immutable bytecode, a mutex-protected **ready queue** of suspended tasks paired with a **condition variable** so idle workers block instead of spinning, the cooperative **timer queue**, **task id** allocation, **task result** storage for handles used with `Wait`, a **shutdown** flag, and mutex-protected **console**, **input**, and **TUI** state so concurrent tasks do not corrupt I/O. Hosted TUI `On*` handlers run on the **main** thread only.

## See also

- [Concurrency overview](README.md) — bytecode mapping for retained vs detached spawn
- [`go`](go.md)
- [Task handles](task-handles.md)
