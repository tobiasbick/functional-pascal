# Parallel VM Execution — Implemented

The VM uses true multi-threaded parallel execution for `go` tasks. No language changes were required.

## Architecture

### Worker / SharedState Split

The monolithic `Vm` struct is split into two concerns:

- **`Worker`** — per-thread execution state: instruction pointer, stack, call stack, current task ID, timeslice counter. Each OS thread owns exactly one `Worker`.
- **`SharedState`** — `Arc`-wrapped resources shared across all workers: bytecode chunk, globals (`RwLock`), channels (`Mutex<HashMap>`), task queue (`Mutex<Vec>` + `Condvar`), task results (`Mutex<HashMap>`), console/input (`Mutex`), atomic counters for IDs, and an `AtomicBool` shutdown flag.

### Thread Pool

- Pool size = `available_parallelism() - 1` (main thread counts as one worker).
- Uses `std::thread::scope` — all workers share the `Arc<SharedState>`.
- Pool workers run `pool_loop()`: dequeue tasks, execute them, store results, wait on `Condvar` when idle.
- Main worker executes task 0 (the program entry point) directly.

### Task Lifecycle

1. `go Function(args)` creates a `TaskState` (id, ip, stack snapshot, call stack) and enqueues it via `SharedState::enqueue_task()`.
2. Any idle worker dequeues the task, loads it with `Worker::load_task()`, and executes until completion or yield.
3. On completion, the result is stored via `SharedState::store_task_result()` and all workers are notified.
4. `Wait(T)` checks `SharedState::take_task_result()`; if not ready, the task yields and retries.

### Channels

Channels use `crossbeam-channel::bounded()` for lock-free MPMC communication:

- `Send` uses `try_send()` — if the buffer is full, the task yields and retries (no spin-wait).
- `Receive` uses `try_recv()` — if empty, the task yields and retries.
- `Close` sets an `AtomicBool` flag on the channel; subsequent sends error, receives drain remaining values.
- Channel metadata (sender, receiver, closed flag) is stored in a `Mutex<HashMap<u64, SharedChannel>>`.

### Scheduling

- Each worker has a 256-instruction timeslice (`TIMESLICE` constant).
- On timeslice expiry or explicit yield, the current task is saved and enqueued; the worker picks the next task from the shared queue.
- If no tasks are queued, the worker continues the current task (no-op yield).
- Pool workers that have no task wait on a `Condvar` until a new task is enqueued or shutdown is signalled.

### Globals

- Global variables use `RwLock<HashMap<String, Value>>` — reads are concurrent, writes are exclusive.
- `GetGlobal` acquires a read lock; `SetGlobal` / `DefineGlobal` acquire a write lock.

### Console / I/O

- `Console`, `TextInput`, and `KeyInput` are each wrapped in their own `Mutex`.
- I/O operations lock only the specific mutex they need, avoiding contention between unrelated operations.

## Non-Goals

- Shared mutable state between tasks (no mutexes or locks in the language).
- Explicit thread control from user code.
- Async I/O — the VM handles I/O scheduling internally.
