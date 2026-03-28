# Parallel VM Execution

Automatic multi-core execution for `go` tasks — no language changes required.

## Current State

The VM uses **cooperative scheduling** on a single OS thread. Tasks created with `go` run in round-robin with a 256-instruction timeslice. Concurrency is real (channels, select, wait all work), but there is no parallelism.

## Goal

The VM decides autonomously how many OS threads to use. User code stays identical — `go Worker()` means "this can run concurrently", the VM decides *how*.

## Design

### Thread Pool

- One worker thread per CPU core (configurable, like Go's `GOMAXPROCS`)
- Global task queue with work-stealing: idle workers take tasks from busy workers
- Main thread runs the program entry point; `go` pushes tasks to the global queue

### Value Ownership

Current `Value` types use shared references. For multi-threaded execution:

- **Deep-copy on channel send** — values crossing thread boundaries must be independent
- **Move semantics for `go` arguments** — task arguments are copied into the new task's stack frame
- Immutable-by-default makes this safe: most values are never mutated after creation

### Thread-Safe Channels

Replace `VecDeque`-based channel buffer with a concurrent queue:

- `crossbeam-channel` or similar lock-free MPMC queue
- Send/receive operations must not require the VM's global lock
- Blocking operations yield the current task back to the thread pool, not spin-wait

### Task Scheduling

| Scenario | Behavior |
|----------|----------|
| Few short tasks | Run on single thread (no overhead) |
| Many independent tasks | Distribute across all cores |
| Task blocked on channel | Yield to pool, resume when data arrives |
| `Std.Task.Wait(T)` | If T is on same thread, inline-execute; otherwise block and steal work |

### Migration Path

1. Make `Value` either `Clone` or `Arc`-wrapped for cross-thread sharing
2. Replace channel internals with `crossbeam-channel`
3. Add a thread pool (e.g., `rayon` or custom work-stealing pool)
4. Distribute `go`-spawned tasks across pool workers
5. Keep single-threaded mode as fallback (pool size = 1)

## Non-Goals

- Shared mutable state between tasks (no mutexes, no locks in the language)
- Explicit thread control from user code
- Async I/O — the VM handles I/O scheduling internally
