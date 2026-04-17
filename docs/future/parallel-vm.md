# Parallel stack VM — implementation roadmap

This document is for **Rust contributors** who extend or port the bytecode VM’s task runtime. It does **not** restate the language rules for `go`, `task`, `Wait`, or `WaitAll` — those live in [`docs/pascal/08-concurrency.md`](../pascal/08-concurrency.md) and [`docs/pascal/std/task.md`](../pascal/std/task.md).

Follow the phases **in order** when building or auditing the system. Each phase lists a **goal**, **primary crates/paths**, and **how to know you are done**.

### Status in this repository

| Phase | State |
|-------|--------|
| **1** — Bytecode surface | **Done** — opcodes and `Chunk::uses_spawn_tasks` in `fpas-bytecode`; VM reads the flag at construction (`fpas-vm`). |
| **2** — Compiler lowering | **Done** — `go` expression vs statement map to `Op::SpawnTask` / `Op::SpawnDetachedTask` in `fpas-compiler`. |
| **3** — Shared state, queues, I/O | **Done** — `SharedState` in `fpas-vm` (`Arc` at runtime): chunk, globals, ready queue + condvar, task ids/results, shutdown flag, console and input/TUI mutexes; lock-ordering notes in source. Tests: `crates/fpas-vm/src/tests/shared_state.rs`. |
| **4** — Conditional pool, scoped `run` | **Done** — pool size `0` when `Chunk::uses_spawn_tasks()` is false, else `max(1, available_parallelism − 1)`; `Vm::run` uses `thread::scope`, main task on caller thread, `SharedState::request_shutdown` (`notify_all`) after main. Tests: `crates/fpas-vm/src/tests/worker_pool.rs`. |
| **5** — Worker pull loop, task binding | **Done** — main worker (task `0`), pool sentinel (`u64::MAX`), `pool_loop` fast dequeue + condvar wait, `TaskState` save/load. Tests: `crates/fpas-vm/src/tests/pool_worker_loop.rs`. |
| **6** — Execute — spawn path | **Done** — callee + args popped per opcode arity, chunk entry resolved, [`TaskState`](../../crates/fpas-vm/src/vm/shared.rs) enqueued, retained spawn pushes `Value::Task`. Tests: `crates/fpas-vm/src/tests/spawn_path.rs`. |
| **7** — Cooperative scheduling (yield / timeslice) | **Done** — `Op::Yield` and a fixed instruction budget (`TIMESLICE` in `fpas-vm`) reschedule **spawned** tasks via save/enqueue/dequeue; main task (`0`) never enters the shared queue (`thread::yield_now` on main `Yield`). Tests: `crates/fpas-vm/src/tests/yield_scheduling.rs`. |
| **8** — Blocking wait intrinsics | **Done** — `Std.Task.Wait` / `WaitAll` retry with cooperative `Yield` on the main task, then block on the shared condvar (no hot-spin); `store_task_result` notifies waiters. Tests: `crates/fpas-vm/src/tests/wait_blocking.rs`, `shared_state.rs` (condvar). |
| **9** — Errors, shutdown, harness | **Done** — runtime failure sets `SharedState::signal_runtime_failure` (shutdown + `abort_spawned_bytecode`); normal main completion uses `request_shutdown` only so queued tasks still run; `Vm::run` guard drops [`ShutdownAfterMain`](../../crates/fpas-vm/src/vm/mod.rs) after the main worker returns or unwinds; pool join prefers the main task’s diagnostic when both fail; pool panic → internal error. Tests: [`runtime_shutdown.rs`](../../crates/fpas-vm/src/tests/runtime_shutdown.rs), [`spawn_path.rs`](../../crates/fpas-vm/src/tests/spawn_path.rs), [`tasks.rs`](../../crates/fpas-vm/src/tests/tasks.rs), [`wait_blocking.rs`](../../crates/fpas-vm/src/tests/wait_blocking.rs), [`pool_shutdown.rs`](../../crates/fpas-vm/src/tests/pool_shutdown.rs). |

---

## Phase 1: Bytecode surface

**Goal:** The VM must see explicit spawn and yield operations; the host must be able to decide **at VM construction time** whether a thread pool is needed.

**Do:**

1. Define stack-machine opcodes for **retained spawn** (push a task handle), **detached spawn** (discard result), and **cooperative yield** (e.g. `Op::SpawnTask`, `Op::SpawnDetachedTask`, `Op::Yield`) in `crates/fpas-bytecode`.
2. Add a **static scan** over emitted code (e.g. `Chunk::uses_spawn_tasks`) that is `true` iff any spawn opcode appears anywhere in the chunk.

**Done when:** The compiler can emit these ops and the VM can query `uses_spawn_tasks` without executing the program.

**Implemented:** [`Op`](../../crates/fpas-bytecode/src/op.rs) in `crates/fpas-bytecode/src/op.rs`; scan in [`Chunk::uses_spawn_tasks`](../../crates/fpas-bytecode/src/chunk.rs) over `Chunk::code` only — it treats **`Op::SpawnTask` and `Op::SpawnDetachedTask`** as spawns. **`Op::Yield` does not set the flag** (yield is for scheduling only; pool sizing follows spawn opcodes). Integration tests: `crates/fpas-bytecode/tests/parallel_vm_phase1.rs`.

---

## Phase 2: Compiler lowering

**Goal:** Source-level `go` maps deterministically onto Phase 1 opcodes.

**Do:**

1. **Expression** `go callee(args)` → opcode that **retains** the task result for `Wait`.
2. **Statement** `go callee(args);` → opcode that **does not** retain the result (fire-and-forget).
3. Keep rejection of non-call `go` operands in the parser/sema; lowering only handles valid calls (including wrappers the compiler emits for `Std.*` calls if applicable).

**Primary path:** `crates/fpas-compiler` — task spawn lowering (see module that lowers `go` statements and expressions).

**Done when:** Every program that uses `go` produces a chunk that passes Phase 1’s scan; programs without `go` produce chunks where the scan is false.

**Implemented:** Statement lowering [`compiler/stmt/concurrency.rs`](../../crates/fpas-compiler/src/compiler/stmt/concurrency.rs) (`Stmt::Go` → detached spawn); expression lowering [`compiler/expr/mod.rs`](../../crates/fpas-compiler/src/compiler/expr/mod.rs) (`Expr::Go` → retained spawn). Compiler tests: `crates/fpas-compiler/src/tests/parallel_vm_phase1.rs`.

---

## Phase 3: Shared state, queues, and I/O

**Goal:** All threads that touch the VM share one **`Arc<SharedState>`** with clear locking rules.

**Do:**

1. Hold the immutable **`Chunk`** in shared state (read-only after load).
2. Add a **ready queue** of saved task frames (mutex-protected list or deque).
3. Add a **condition variable** paired with that mutex so idle workers block instead of spinning when the queue is empty.
4. Add **task id allocation**, **task result storage** (for handles that await a return value), and an **atomic shutdown flag** (or equivalent) so the main task can stop the pool cleanly.
5. Route **console**, **line/text input**, **key input**, and **TUI session** state through the same shared object behind **mutexes** (or one mutex per concern) so pool workers and the main worker do not corrupt I/O.

**Mutex discipline (for later features such as a Rust-hosted TUI loop):** Treat `SharedState` as the single place that defines **which fields are behind which lock**. Any new intrinsic or host bridge must **document** lock ordering if it takes more than one lock; avoid calling into user bytecode while holding locks you do not control.

**Primary path:** `crates/fpas-vm/src/vm/shared.rs`.

**Done when:** Types compile, fields exist, and enqueue/dequeue/wait helpers are callable from the worker without data races (under Miri or careful review).

**Implemented:** [`SharedState`](../../crates/fpas-vm/src/vm/shared.rs) in `crates/fpas-vm/src/vm/shared.rs` (chunk, `globals` `RwLock`, `task_queue` + `task_available`, `task_results`, `next_task_id`, `shutdown`, `console`, `text_input`, `key_input`, `tui`). [`Worker::pool_loop`](../../crates/fpas-vm/src/vm/worker.rs) uses the queue mutex with the condition variable when the fast dequeue path finds an empty queue. VM tests: `crates/fpas-vm/src/tests/shared_state.rs`.

---

## Phase 4: VM process shape — conditional pool and scoped lifetime

**Goal:** Only programs that **need** workers pay for them; all pool threads **outlive** only one `Vm::run` invocation.

**Do:**

1. In **VM build**, set worker count to **zero** when `uses_spawn_tasks()` is false; otherwise set pool size to **`max(1, available_parallelism − 1)`** (or the project’s chosen policy — keep in sync with [`08-concurrency.md`](../pascal/08-concurrency.md)).
2. In **`Vm::run`**, use a **scoped** thread API so pool threads are joined before `run` returns. Start pool workers first (or start them and immediately run the main worker per your ordering), then run **task 0** on the **caller’s OS thread**.
3. When the main task finishes or aborts, set **shutdown** and wake **all** waiters on the condition variable so pool threads exit.

**Primary path:** `crates/fpas-vm/src/vm/mod.rs`.

**Done when:** Thousands of short-lived VMs without `go` do not leave idle worker threads; programs with `go` still terminate and join workers reliably.

**Implemented:** [`Vm::build`](../../crates/fpas-vm/src/vm/mod.rs) / [`Vm::run`](../../crates/fpas-vm/src/vm/mod.rs) (`thread::scope`, pool threads then main worker, shutdown after main); [`SharedState::request_shutdown`](../../crates/fpas-vm/src/vm/shared.rs) (`notify_all`). VM tests: `crates/fpas-vm/src/tests/worker_pool.rs`.

---

## Phase 5: Worker threads — pull loop and task binding

**Goal:** Each OS thread runs **at most one** FPAS task at a time, with a clear distinction between **main** and **pool** workers.

**Do:**

1. Implement a **main worker** constructor (task id `0`, initial IP `0`).
2. Implement a **pool worker** constructor (sentinel “no task loaded” until dequeue).
3. **`pool_loop`:** try fast dequeue; if empty and not shutdown, **lock** the queue mutex and **`wait`** on the condition variable; on wakeup, pop work or exit on shutdown.
4. Provide **load/save** of `TaskState` (IP, stacks, call frames, retain-result flag) so tasks can migrate between threads.

**Primary path:** `crates/fpas-vm/src/vm/worker.rs`.

**Done when:** Pool workers block on an empty queue and wake when `enqueue_task` runs `notify_one` (or equivalent).

**Implemented:** [`Worker::new_main`](../../crates/fpas-vm/src/vm/worker.rs), [`Worker::new_pool`](../../crates/fpas-vm/src/vm/worker.rs), [`Worker::pool_loop`](../../crates/fpas-vm/src/vm/worker.rs) (fast `try_dequeue_task`, then lock `task_queue` and wait on `task_available` when idle), [`Worker::load_task`](../../crates/fpas-vm/src/vm/worker.rs) / [`Worker::save_task`](../../crates/fpas-vm/src/vm/worker.rs) over [`TaskState`](../../crates/fpas-vm/src/vm/shared.rs). VM tests: `crates/fpas-vm/src/tests/pool_worker_loop.rs`.

---

## Phase 6: Execute — spawn path

**Goal:** Spawning pushes real work onto the ready queue and assigns stable task ids.

**Do:**

1. Pop callee and arguments per opcode arity; resolve the target function entry in the chunk.
2. Allocate a new task id; build initial `TaskState`; **enqueue** on the shared queue.
3. For retained spawn, push **`Value::Task(id)`** onto the spawning task’s stack; for detached spawn, omit the handle.

**Primary path:** `crates/fpas-vm/src/vm/execute/concurrency/` (spawn helpers).

**Done when:** Integration tests can run `go` functions that return values and observe them via `Wait`.

**Implemented:** [`Worker::exec_spawn_task`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/spawn.rs) (`retained` vs detached via opcode dispatch in [`concurrency/mod.rs`](../../crates/fpas-vm/src/vm/execute/concurrency/mod.rs)); task id from [`SharedState::alloc_task_id`](../../crates/fpas-vm/src/vm/shared.rs); enqueue via [`SharedState::enqueue_task`](../../crates/fpas-vm/src/vm/shared.rs). Top-level return stores results for waiters in [`Worker::run`](../../crates/fpas-vm/src/vm/execute/mod.rs). VM tests: [`spawn_path.rs`](../../crates/fpas-vm/src/tests/spawn_path.rs) (also [`tasks.rs`](../../crates/fpas-vm/src/tests/tasks.rs) for wait edge cases).

---

## Phase 7: Cooperative scheduling (yield / timeslice)

**Goal:** Long-running tasks **relinquish** the CPU so other tasks and the main thread make progress.

**Do:**

1. After a bounded number of instructions (timeslice), or on **`Op::Yield`**, save the current task and **enqueue** it again (unless rules forbid re-queuing the main task — follow the existing invariants in the VM).
2. Ensure the **main task** never competes incorrectly for the ready queue (document in code: main runs on caller thread; only **spawned** tasks should appear in the pool queue).

**Primary path:** `crates/fpas-vm/src/vm/execute/concurrency/tasks/scheduling.rs` and opcode dispatch for `Yield` in [`concurrency/mod.rs`](../../crates/fpas-vm/src/vm/execute/concurrency/mod.rs); cooperative `Yield` / `switch_to_next_ready_task` in [`tasks/mod.rs`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/mod.rs).

**Done when:** Busy spawned tasks still allow the main program to reach `Wait` / completion without starvation in tests.

**Implemented:** Instruction budget in [`Worker::maybe_timeslice_yield`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/scheduling.rs) (skips while `sync_call_depth > 0`, skips enqueue for task id `0`). [`Worker::exec_yield`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/mod.rs): main uses `std::thread::yield_now`, spawned tasks use [`Worker::switch_to_next_ready_task`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/mod.rs). Budget reset on [`Worker::load_task`](../../crates/fpas-vm/src/vm/worker.rs). VM tests: [`yield_scheduling.rs`](../../crates/fpas-vm/src/tests/yield_scheduling.rs).

---

## Phase 8: Blocking wait intrinsics

**Goal:** `Std.Task.Wait` and `Std.Task.WaitAll` block **without** busy-spinning when no progress is possible.

**Do:**

1. **`Wait`:** If the result is not ready, re-push the task handle (or rewind IP), run **yield** logic, then **wait** on the shared condition variable (unbounded wait or a bounded timeout for tests) until task completion or shutdown; wakeups also come from `enqueue_task`, `store_task_result`, and shutdown.
2. **`WaitAll`:** Barrier over an **array of task handles**; must not consume results if the language spec says so (see [`std/task.md`](../pascal/std/task.md)).
3. Map failed child tasks to a single **shutdown / aborted** error path for waiters.

**Primary path:** `crates/fpas-vm/src/vm/execute/concurrency/tasks/wait.rs` and intrinsic dispatch in `execute/concurrency/mod.rs`.

**Done when:** Behavior matches `docs/pascal/std/task.md` (double-wait errors, empty `WaitAll`, etc.).

**Implemented:** [`Worker::exec_task_wait`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/wait.rs) / [`Worker::exec_task_wait_all`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/wait.rs): loop with [`Worker::exec_yield`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/mod.rs), then [`SharedState::wait_for_task_progress`](../../crates/fpas-vm/src/vm/shared.rs) with `None` for unbounded condvar wait; [`SharedState::store_task_result`](../../crates/fpas-vm/src/vm/shared.rs) calls `notify_all` so completion wakes waiters. VM tests: [`wait_blocking.rs`](../../crates/fpas-vm/src/tests/wait_blocking.rs); condvar unit tests in [`shared_state.rs`](../../crates/fpas-vm/src/tests/shared_state.rs).

---

## Phase 9: Errors, shutdown, and test harness

**Goal:** One primary diagnostic surfaces to the user; background threads cannot deadlock the process on panic or runtime error.

**Do:**

1. On **runtime error** in any worker, signal **global shutdown** and propagate an error to `Vm::run` (define whether main or pool error wins if both fail).
2. **`join`** pool threads and translate panics into an internal-error diagnostic.
3. Add **VM unit tests** for spawn arity errors, wait semantics, and pool shutdown (see existing tests under `crates/fpas-vm/src/tests/`).

**Done when:** `cargo test --workspace` passes and stress scenarios (many VMs, many tasks) remain bounded in thread count.

**Implemented:**

- **Shutdown vs abort:** [`SharedState::request_shutdown`](../../crates/fpas-vm/src/vm/shared.rs) wakes idle pool workers when the **main task** has finished its `Worker::run` (success or failure); it does **not** stop bytecode that is already scheduled on the ready queue so detached work can still run before `Vm::run` joins the pool. A **runtime failure** in any worker calls [`SharedState::signal_runtime_failure`](../../crates/fpas-vm/src/vm/shared.rs), which sets **`abort_spawned_bytecode`** and shutdown so other **spawned** tasks exit cooperatively at the next [`Worker::run`](../../crates/fpas-vm/src/vm/execute/mod.rs) loop boundary (main task id `0` still uses [`Worker::check_shutdown`](../../crates/fpas-vm/src/vm/execute/mod.rs) for `RUNTIME_VM_SHUTDOWN`).
- **`Vm::run`:** [`ShutdownAfterMain`](../../crates/fpas-vm/src/vm/mod.rs) ensures [`SharedState::request_shutdown`](../../crates/fpas-vm/src/vm/shared.rs) runs after the main worker returns **or unwinds**, so pool threads do not stay blocked on an empty queue; each `run` clears **`abort_spawned_bytecode`**. Join order: if the main task returned **`Err`**, that diagnostic wins; otherwise the first pool **`Err`** is returned; pool **`join` panics** become an internal diagnostic.
- **Pool loop:** [`Worker::pool_loop`](../../crates/fpas-vm/src/vm/worker.rs) re-tries `try_dequeue_task` once when shutdown is already true so a task enqueued immediately before teardown is not skipped by a stale empty fast-path check.

---

## Optional extensions (not specified in the Pascal docs)

Pick these only if the project explicitly adopts them; they are **not** required for conformance to [`08-concurrency.md`](../pascal/08-concurrency.md).

- **Work stealing** or multiple queues instead of one global ready queue.
- **Task priorities** or fair scheduling beyond a fixed instruction timeslice.
- **Affinity** or pinning long tasks to threads.
- **Structured concurrency** (scoped tasks tied to lexical blocks) as a language change.

---

## Quick file index (current layout)

| Concern | Location |
|---------|----------|
| VM entry, pool size, scoped `run` | `crates/fpas-vm/src/vm/mod.rs` |
| Pool loop, main vs pool worker | `crates/fpas-vm/src/vm/worker.rs` |
| Shared queues, condvar, I/O mutexes | `crates/fpas-vm/src/vm/shared.rs` |
| Spawn / yield / intrinsic dispatch | `crates/fpas-vm/src/vm/execute/concurrency/` |
| Spawn detection on chunk | `crates/fpas-bytecode/src/chunk.rs` (`Chunk::uses_spawn_tasks`) |
| Phase 1–2 tests (bytecode / compiler / VM) | `crates/fpas-bytecode/tests/parallel_vm_phase1.rs`, `crates/fpas-compiler/src/tests/parallel_vm_phase1.rs`, `crates/fpas-vm/src/tests/uses_spawn_tasks.rs` |
| Shared-state tests (queue / I/O mutexes) | `crates/fpas-vm/src/tests/shared_state.rs` |
| Phase 4 tests (pool sizing, scoped run, shutdown / condvar) | `crates/fpas-vm/src/tests/worker_pool.rs` |
| Phase 5 tests (pool loop, save/load, enqueue wake, errors) | `crates/fpas-vm/src/tests/pool_worker_loop.rs` |
| Phase 6 tests (spawn execute path: arity, captures, detached, errors) | `crates/fpas-vm/src/tests/spawn_path.rs` |
| Phase 7 tests (yield, timeslice, main vs pool queue) | `crates/fpas-vm/src/tests/yield_scheduling.rs` |
| Phase 8 tests (Wait / WaitAll blocking, errors) | `crates/fpas-vm/src/tests/wait_blocking.rs` |
| Phase 9 tests (shutdown vs abort, `Vm::run` errors, join / panic) | `crates/fpas-vm/src/tests/runtime_shutdown.rs` |
| `go` lowering (retained vs detached) | `crates/fpas-compiler/src/compiler/stmt/concurrency.rs`, `crates/fpas-compiler/src/compiler/expr/mod.rs` |
