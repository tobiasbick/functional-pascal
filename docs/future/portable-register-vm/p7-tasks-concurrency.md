# P7 tasks and concurrency implementation

P7 is complete on the inactive register-development path. Production `fpas run` still uses the
stack VM until P9, so this phase changes no FPAS syntax, semantics, or current user-facing runtime
selection.

## Implemented ownership

- `fpas-compiler/lowering/concurrency.rs` lowers both existing `go` forms to typed task IR, preserves
  source-order argument evaluation, and marks spawning functions for verifier admission.
- `fpas-compiler/bytecode/selection.rs` emits the fixed retained and detached register windows.
- `fpas-vm/vm/register/tasks/state.rs` saves task ID, numeric function ID, instruction address,
  register store, active frame stack, base register, retain-result state, and instruction count.
- `tasks/scheduler.rs` owns the FIFO ready queue, retained result registry, completion records,
  cooperative timer queue, cancellation, and shutdown flags. Immutable verified executable metadata
  remains shared separately through `Arc<VerifiedExecutable>`.
- `tasks/pool.rs` drives transferable task states on OS workers and lets a waiting worker execute one
  ready task inline, preventing a one-worker parent/child wait deadlock.
- the pre-existing timer queue is generic over task state and is shared by both interpreters; its
  cancellation and monotonic-deadline policy is not duplicated.

The scalar dispatch path performs only local register and counter operations. Queue/result/timer
locks occur only on spawn, suspension, Wait/WaitAll, timer handoff, completion, and shutdown.
Mutable capture cells remain `Arc<Mutex<Value>>`; closure construction propagates the task-bound bit,
and spawn rejects such closures before making them visible to a worker.

## Preserved behavior

Retained task handles are registered before enqueue, successful results are consumed by `Wait`, and
`WaitAll` observes without consuming them. Detached work returns no handle. Spawned
`Std.Time.Sleep` saves the complete register task and releases its worker until the timer driver
requeues it. Main-task sleep remains the ordinary blocking intrinsic. Timeslicing saves after 256
logical instructions; main-task `Yield` only yields the OS thread. A cloneable register shutdown
handle cancels main and spawned execution at instruction boundaries.

## Verification evidence

`register_subset::concurrency` covers retained and detached spawn, Wait, WaitAll result retention,
cooperative sleep, task-bound mutable captures, and suspension inside a nested call while an
aggregate remains live in its caller. Direct register tests cover Yield, task operand diagnostics,
and shutdown cancellation. The existing stack-VM concurrency, pool, shutdown, sleep, panic, and
stress suites remain unchanged and pass as regression protection.

The `register-p7` benchmark group runs a focused two-task `register_task_spawn_wait` smoke workload
through `engine = "register"`; the larger production row remains a P9 cutover gate.
Because the register path is not production and had no prior runnable task baseline, this is
exercised-path evidence only; it is not compared to stack timings and makes no speedup claim.

## Deferred boundary

Unit objects, linker relocation, portable artifact encoding, CLI selection, old stack-path removal,
and production performance acceptance remain P8 through P10 work.
