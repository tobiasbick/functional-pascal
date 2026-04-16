# Parallel stack VM — implementation roadmap

This document is for **Rust contributors** who extend or port the bytecode VM’s task runtime. It does **not** restate the language rules for `go`, `task`, `Wait`, or `WaitAll` — those live in [`docs/pascal/08-concurrency.md`](../pascal/08-concurrency.md) and [`docs/pascal/std/task.md`](../pascal/std/task.md).

Follow the phases **in order** when building or auditing the system. Each phase lists a **goal**, **primary crates/paths**, and **how to know you are done**.

---

## Phase 1: Bytecode surface

**Goal:** The VM must see explicit spawn and yield operations; the host must be able to decide **at VM construction time** whether a thread pool is needed.

**Do:**

1. Define stack-machine opcodes for **retained spawn** (push a task handle), **detached spawn** (discard result), and **cooperative yield** (e.g. `Op::SpawnTask`, `Op::SpawnDetachedTask`, `Op::Yield`) in `crates/fpas-bytecode`.
2. Add a **static scan** over emitted code (e.g. `Chunk::uses_spawn_tasks`) that is `true` iff any spawn opcode appears anywhere in the chunk.

**Done when:** The compiler can emit these ops and the VM can query `uses_spawn_tasks` without executing the program.

---

## Phase 2: Compiler lowering

**Goal:** Source-level `go` maps deterministically onto Phase 1 opcodes.

**Do:**

1. **Expression** `go callee(args)` → opcode that **retains** the task result for `Wait`.
2. **Statement** `go callee(args);` → opcode that **does not** retain the result (fire-and-forget).
3. Keep rejection of non-call `go` operands in the parser/sema; lowering only handles valid calls (including wrappers the compiler emits for `Std.*` calls if applicable).

**Primary path:** `crates/fpas-compiler` — task spawn lowering (see module that lowers `go` statements and expressions).

**Done when:** Every program that uses `go` produces a chunk that passes Phase 1’s scan; programs without `go` produce chunks where the scan is false.

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

---

## Phase 4: VM process shape — conditional pool and scoped lifetime

**Goal:** Only programs that **need** workers pay for them; all pool threads **outlive** only one `Vm::run` invocation.

**Do:**

1. In **VM build**, set worker count to **zero** when `uses_spawn_tasks()` is false; otherwise set pool size to **`max(1, available_parallelism − 1)`** (or the project’s chosen policy — keep in sync with [`08-concurrency.md`](../pascal/08-concurrency.md)).
2. In **`Vm::run`**, use a **scoped** thread API so pool threads are joined before `run` returns. Start pool workers first (or start them and immediately run the main worker per your ordering), then run **task 0** on the **caller’s OS thread**.
3. When the main task finishes or aborts, set **shutdown** and wake **all** waiters on the condition variable so pool threads exit.

**Primary path:** `crates/fpas-vm/src/vm/mod.rs`.

**Done when:** Thousands of short-lived VMs without `go` do not leave idle worker threads; programs with `go` still terminate and join workers reliably.

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

---

## Phase 6: Execute — spawn path

**Goal:** Spawning pushes real work onto the ready queue and assigns stable task ids.

**Do:**

1. Pop callee and arguments per opcode arity; resolve the target function entry in the chunk.
2. Allocate a new task id; build initial `TaskState`; **enqueue** on the shared queue.
3. For retained spawn, push **`Value::Task(id)`** onto the spawning task’s stack; for detached spawn, omit the handle.

**Primary path:** `crates/fpas-vm/src/vm/execute/concurrency/` (spawn helpers).

**Done when:** Integration tests can run `go` functions that return values and observe them via `Wait`.

---

## Phase 7: Cooperative scheduling (yield / timeslice)

**Goal:** Long-running tasks **relinquish** the CPU so other tasks and the main thread make progress.

**Do:**

1. After a bounded number of instructions (timeslice), or on **`Op::Yield`**, save the current task and **enqueue** it again (unless rules forbid re-queuing the main task — follow the existing invariants in the VM).
2. Ensure the **main task** never competes incorrectly for the ready queue (document in code: main runs on caller thread; only **spawned** tasks should appear in the pool queue).

**Primary path:** `crates/fpas-vm/src/vm/execute/concurrency/tasks/scheduling.rs` and opcode dispatch for `Yield`.

**Done when:** Busy spawned tasks still allow the main program to reach `Wait` / completion without starvation in tests.

---

## Phase 8: Blocking wait intrinsics

**Goal:** `Std.Task.Wait` and `Std.Task.WaitAll` block **without** busy-spinning when no progress is possible.

**Do:**

1. **`Wait`:** If the result is not ready, re-push the task handle (or rewind IP), run **yield** logic, and optionally **wait** on the shared condition variable with a short timeout until task completion or shutdown.
2. **`WaitAll`:** Barrier over an **array of task handles**; must not consume results if the language spec says so (see [`std/task.md`](../pascal/std/task.md)).
3. Map failed child tasks to a single **shutdown / aborted** error path for waiters.

**Primary path:** `crates/fpas-vm/src/vm/execute/concurrency/tasks/wait.rs` and intrinsic dispatch in `execute/concurrency/mod.rs`.

**Done when:** Behavior matches `docs/pascal/std/task.md` (double-wait errors, empty `WaitAll`, etc.).

---

## Phase 9: Errors, shutdown, and test harness

**Goal:** One primary diagnostic surfaces to the user; background threads cannot deadlock the process on panic or runtime error.

**Do:**

1. On **runtime error** in any worker, signal **global shutdown** and propagate an error to `Vm::run` (define whether main or pool error wins if both fail).
2. **`join`** pool threads and translate panics into an internal-error diagnostic.
3. Add **VM unit tests** for spawn arity errors, wait semantics, and pool shutdown (see existing tests under `crates/fpas-vm/src/tests/`).

**Done when:** `cargo test --workspace` passes and stress scenarios (many VMs, many tasks) remain bounded in thread count.

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
| Spawn detection on chunk | `crates/fpas-bytecode/src/chunk.rs` |
