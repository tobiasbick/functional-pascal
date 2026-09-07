# Scheduling

## Thread pool

The runtime starts background workers when any compiled function's verified metadata indicates a
task-start operation: a retained or detached `go` spawn, `Std.Task.StartTaskInGroup`, or
`Std.Task.StartSupervisedTask`. Without these operations, it does not start a worker pool.
Creating a channel or task group, waiting, and `Yield` alone do not activate it.

When required, the runtime starts **`max(1, available_parallelism − 1)`** worker threads that share a ready queue, while the **main task** (task id `0`) runs on the thread that started execution. Each pool thread runs **at most one** ready task at a time: workers block when the queue is empty and are woken when work is enqueued or the runtime shuts down. Together, this matches typical machine parallelism without starting idle workers for programs that never spawn tasks.

Background workers exist only for a single program run: the runtime **joins** pool threads before execution returns so short-lived hosts do not accumulate stray threads across many runs.

Shutdown notifications synchronize with both the ready-queue and timer-queue
mutexes. A worker or timer driver entering its wait during teardown still observes
shutdown and exits; the notification cannot pass between its state check and wait.

When the **main task** finishes, the runtime begins **teardown shutdown**. Idle workers wake and exit after draining tasks that were already in the ready queue. Spawned tasks that are still suspended in `Std.Time.Sleep`, and ready tasks that try to sleep after teardown has begun, are canceled instead of delaying program exit. A retained canceled task is completed with a shutdown error; code that needs its result must call `Wait` before the main task finishes. Detached sleeping tasks are canceled without a result because they have no handle.

Teardown is separate from **task failure**: when a task outside an explicit group aborts with a
runtime error, other spawned work may be stopped cooperatively at the next instruction boundary.
The host surfaces **one** primary diagnostic: if the main task failed, that error wins; otherwise a
worker error is reported. Explicit groups instead retain their children's failures for close;
supervised workers expose only the final outcome after their selected retry policy. An explicit
`Wait` or `WaitAll` still propagates a failed task's diagnostic. See [Std.Task](../../std/concurrency/task.md#task-groups).

Group close is cooperative. `CloseTaskGroupWithTimeout` can end a waiting attempt without releasing
unfinished children; it does not force workers to terminate or bound VM teardown. Blocking host
calls can delay shutdown. Keep the VM alive for required cleanup and retry closing timed-out groups.

## Cooperative scheduling

Spawned tasks can be **preempted cooperatively** after a fixed instruction budget and on the **`Yield`** opcode so long-running bytecode cannot starve other queued tasks on the same worker. The shared ready queue is **FIFO**: the oldest suspended task is resumed first. The **main** program task always runs on the thread that started execution and is **not** placed on the shared ready queue; a main-thread `Yield` yields the OS thread so pool workers can run.

`Std.Time.Sleep` is also a cooperative suspension point for spawned tasks. Sleeping tasks are grouped
by millisecond deadline in a shared timer queue. One timer-driver thread moves each due group to the
ready queue, so sleeping tasks do not occupy pool workers. `Sleep` on the main task remains a blocking
host wait.

Child task waits, blocking channel operations, mixed-source selection, and group close also save
pending operation state and release the pool thread. Shared timer probes resume them without
retaining an inline helper's waiting parent stack. Values and deadlines survive suspension.
The main task may help queued work while waiting, except during `CloseTaskGroupWithTimeout`, which
leaves child execution to the pool so it can observe its waiting budget. Requested timer intervals
are not hard wall-clock guarantees; see [Waiting and execution](../../std/concurrency/task.md#waiting-and-execution).

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
