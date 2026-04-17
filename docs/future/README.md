# Future Features

Open planning items for Functional Pascal.

## VM implementation

[`parallel-vm.md`](parallel-vm.md) is an **ordered implementation roadmap** for the parallel task runtime in `fpas-vm` (bytecode through shutdown). Language rules stay in [`docs/pascal/08-concurrency.md`](../pascal/08-concurrency.md). Phases **1** (spawn / yield opcodes + `Chunk::uses_spawn_tasks`), **2** (`go` lowering), **3** (shared `Arc<SharedState>`: ready queue, condvar, task results, I/O mutexes), **4** (conditional worker pool + scoped `Vm::run` + shutdown waking waiters), **5** (pool `Worker` pull loop, main vs pool task binding, `TaskState` save/load), **6** (execute-time spawn: callee resolution, enqueue, `Value::Task` vs detached), and **7** (cooperative `Op::Yield` + instruction timeslice for spawned tasks; main task never on the shared ready queue) are **implemented** in the current tree; see the status table at the top of that file.

## TUI roadmap

Turbo Vision–style direction: Rust-hosted event loop, FPAS `On*` handlers and `RunApp`-style entry, migration away from poll-heavy console usage. See [`tui-application-framework.md`](tui-application-framework.md).

## Under Consideration

| # | Feature | Description |
|---|---------|-------------|
| 9 | [`dict`](09-remove-dict.md) | Pending — may be kept |

## Not Yet Planned

| Feature | Description |
|---------|-------------|
| [Libraries](libraries.md) | Project kind `library`, export rules |
