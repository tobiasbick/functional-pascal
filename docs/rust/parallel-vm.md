# Parallel stack VM — implemented layout

Reference for **Rust contributors**: where the parallel task runtime lives after integration. Language rules for `go`, `task`, `Wait`, and `WaitAll` are in [`docs/pascal/08-concurrency.md`](../pascal/08-concurrency.md) and [`docs/pascal/std/task.md`](../pascal/std/task.md).

## Summary

| Area | What exists |
|------|-------------|
| **1–2** — Opcodes & `go` lowering | `Op::SpawnTask`, `Op::SpawnDetachedTask`, `Op::Yield`; [`Chunk::uses_spawn_tasks`](../../crates/fpas-bytecode/src/chunk.rs) scans **spawn opcodes only** (not `Yield`). Compiler: [`stmt/concurrency.rs`](../../crates/fpas-compiler/src/compiler/stmt/concurrency.rs), [`expr/mod.rs`](../../crates/fpas-compiler/src/compiler/expr/mod.rs). Tests: [`fpas-bytecode` / `parallel_vm_phase1.rs`](../../crates/fpas-bytecode/tests/parallel_vm_phase1.rs), [`fpas-compiler` / `parallel_vm_phase1.rs`](../../crates/fpas-compiler/src/tests/parallel_vm_phase1.rs), [`uses_spawn_tasks.rs`](../../crates/fpas-vm/src/tests/uses_spawn_tasks.rs). |
| **3** — Shared state | [`SharedState`](../../crates/fpas-vm/src/vm/shared.rs): chunk, globals, ready queue + condvar, task ids/results, **`shutdown`**, **`abort_spawned_bytecode`**, console and input/TUI mutexes—elaboration: [Shared state, queues, and I/O (Phase 3)](#phase-3-shared-state-queues-and-io). Tests: [`shared_state.rs`](../../crates/fpas-vm/src/tests/shared_state.rs). |
| **4** — Pool & `Vm::run` | Pool size `0` without spawn bytecode; else `max(1, available_parallelism − 1)`. [`Vm::run`](../../crates/fpas-vm/src/vm/mod.rs): `thread::scope`, pool workers then main task `0` on caller thread. Tests: [`worker_pool.rs`](../../crates/fpas-vm/src/tests/worker_pool.rs). |
| **5** — Workers | [`Worker`](../../crates/fpas-vm/src/vm/worker.rs): `new_main`, `new_pool`, [`pool_loop`](../../crates/fpas-vm/src/vm/worker.rs), `load_task` / `save_task`, [`TaskState`](../../crates/fpas-vm/src/vm/shared.rs). Tests: [`pool_worker_loop.rs`](../../crates/fpas-vm/src/tests/pool_worker_loop.rs). |
| **6** — Spawn execute | [`exec_spawn_task`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/spawn.rs), dispatch in [`concurrency/mod.rs`](../../crates/fpas-vm/src/vm/execute/concurrency/mod.rs). Tests: [`spawn_path.rs`](../../crates/fpas-vm/src/tests/spawn_path.rs), [`tasks.rs`](../../crates/fpas-vm/src/tests/tasks.rs). |
| **7** — Yield / timeslice | [`maybe_timeslice_yield`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/scheduling.rs), [`exec_yield`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/mod.rs). `TIMESLICE` in [`mod.rs`](../../crates/fpas-vm/src/vm/mod.rs). Tests: [`yield_scheduling.rs`](../../crates/fpas-vm/src/tests/yield_scheduling.rs). |
| **8** — Wait / WaitAll | [`wait.rs`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/wait.rs). Tests: [`wait_blocking.rs`](../../crates/fpas-vm/src/tests/wait_blocking.rs). |
| **9** — Shutdown & errors | Teardown: [`SharedState::request_shutdown`](../../crates/fpas-vm/src/vm/shared.rs), [`ShutdownAfterMain`](../../crates/fpas-vm/src/vm/mod.rs). Runtime failure: [`signal_runtime_failure`](../../crates/fpas-vm/src/vm/shared.rs). [`Worker::run`](../../crates/fpas-vm/src/vm/execute/mod.rs) checks **`abort_spawned_bytecode`** for spawned tasks; main uses `check_shutdown`. Pool join prefers main diagnostic if both fail. Tests: [`runtime_shutdown.rs`](../../crates/fpas-vm/src/tests/runtime_shutdown.rs), [`pool_shutdown.rs`](../../crates/fpas-vm/src/tests/pool_shutdown.rs). |

## Module index

| Concern | Location |
|---------|----------|
| VM entry, pool sizing, scoped `run` | [`crates/fpas-vm/src/vm/mod.rs`](../../crates/fpas-vm/src/vm/mod.rs) |
| Pool loop, main vs pool worker | [`crates/fpas-vm/src/vm/worker.rs`](../../crates/fpas-vm/src/vm/worker.rs) |
| Queues, condvar, failure vs teardown | [`crates/fpas-vm/src/vm/shared.rs`](../../crates/fpas-vm/src/vm/shared.rs) |
| Spawn, yield, task intrinsics | [`crates/fpas-vm/src/vm/execute/concurrency/`](../../crates/fpas-vm/src/vm/execute/concurrency/) |
| Spawn scan on bytecode | [`crates/fpas-bytecode/src/chunk.rs`](../../crates/fpas-bytecode/src/chunk.rs) (`Chunk::uses_spawn_tasks`) |
| `go` lowering | [`crates/fpas-compiler/src/compiler/stmt/concurrency.rs`](../../crates/fpas-compiler/src/compiler/stmt/concurrency.rs), [`expr/mod.rs`](../../crates/fpas-compiler/src/compiler/expr/mod.rs) |

<a id="phase-3-shared-state-queues-and-io"></a>

### Shared state, queues, and I/O (Phase 3)

[`SharedState`](../../crates/fpas-vm/src/vm/shared.rs) centralizes chunk data, the ready queue and paired condvar, task ids and wait results, **`shutdown`** (teardown after the main worker finishes) and **`abort_spawned_bytecode`** (cooperative stop after `signal_runtime_failure`), plus mutex-backed console and text/key/TUI state. Lock ordering is documented in the module header—keep new intrinsics consistent with it (relevant when combining pool workers with host-driven UI).
