# Parallel stack VM — implemented layout

Reference for **Rust contributors**: where the parallel task runtime lives after integration. Language rules for `go`, `task`, `Wait`, and `WaitAll` are in [`docs/pascal/08-concurrency.md`](../pascal/08-concurrency.md) and [`docs/pascal/std/task.md`](../pascal/std/task.md).

## Summary

| Area | What exists |
|------|-------------|
| **1–2** — Opcodes & `go` lowering | `Op::SpawnTask`, `Op::SpawnDetachedTask`, `Op::Yield`; [`Chunk::uses_spawn_tasks`](../../crates/fpas-bytecode/src/chunk.rs) scans **spawn opcodes only** (not `Yield`). Compiler: [`stmt/concurrency.rs`](../../crates/fpas-compiler/src/compiler/stmt/concurrency.rs), [`expr/mod.rs`](../../crates/fpas-compiler/src/compiler/expr/mod.rs). Tests: [`fpas-bytecode` / `parallel_vm_phase1.rs`](../../crates/fpas-bytecode/tests/parallel_vm_phase1.rs), [`fpas-compiler` / `parallel_vm_phase1.rs`](../../crates/fpas-compiler/src/tests/parallel_vm_phase1.rs), [`runtime/uses_spawn_tasks.rs`](../../crates/fpas-vm/src/tests/runtime/uses_spawn_tasks.rs). |
| **3** — Shared state | [`SharedState`](../../crates/fpas-vm/src/vm/shared.rs): chunk, globals, ready queue + condvar, task ids/results, **`shutdown`**, **`abort_spawned_bytecode`**, console and input/TUI mutexes—elaboration: [Shared state, queues, and I/O (Phase 3)](#phase-3-shared-state-queues-and-io). TUI host bridge: **`TuiState`**, hosted intrinsics **256**–**347** ([`tui-app.md`](../pascal/std/tui-app.md)). Tests: [`runtime/shared_state_basic.rs`](../../crates/fpas-vm/src/tests/runtime/shared_state_basic.rs), [`core/tui_host_vm/`](../../crates/fpas-vm/src/tests/core/tui_host_vm/mod.rs), [`core/tui_run_vm/`](../../crates/fpas-vm/src/tests/core/tui_run_vm/mod.rs). |
| **4** — Pool & `Vm::run` | Pool size `0` without spawn bytecode; else `max(1, available_parallelism − 1)`. [`Vm::run`](../../crates/fpas-vm/src/vm/mod.rs): `thread::scope`, pool workers then main task `0` on caller thread. Tests: [`pool/worker_pool_run.rs`](../../crates/fpas-vm/src/tests/pool/worker_pool_run.rs). |
| **5** — Workers | [`Worker`](../../crates/fpas-vm/src/vm/worker.rs): `new_main`, `new_pool`, [`pool_loop`](../../crates/fpas-vm/src/vm/worker.rs), `load_task` / `save_task`, [`TaskState`](../../crates/fpas-vm/src/vm/shared.rs). Tests: [`pool/pool_worker_enqueue.rs`](../../crates/fpas-vm/src/tests/pool/pool_worker_enqueue.rs). |
| **6** — Spawn execute | [`exec_spawn_task`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/spawn.rs), dispatch in [`concurrency/mod.rs`](../../crates/fpas-vm/src/vm/execute/concurrency/mod.rs). Tests: [`concurrency/spawn_path_retained_basic.rs`](../../crates/fpas-vm/src/tests/concurrency/spawn_path_retained_basic.rs), [`concurrency/tasks_spawn_wait_errors.rs`](../../crates/fpas-vm/src/tests/concurrency/tasks_spawn_wait_errors.rs). |
| **7** — Yield / timeslice | [`maybe_timeslice_yield`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/scheduling.rs), [`exec_yield`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/mod.rs). `TIMESLICE` in [`mod.rs`](../../crates/fpas-vm/src/vm/mod.rs). Tests: [`concurrency/yield_timeslice.rs`](../../crates/fpas-vm/src/tests/concurrency/yield_timeslice.rs). |
| **8** — Wait / WaitAll | [`wait.rs`](../../crates/fpas-vm/src/vm/execute/concurrency/tasks/wait.rs). Tests: [`concurrency/wait_task.rs`](../../crates/fpas-vm/src/tests/concurrency/wait_task.rs). |
| **9** — Shutdown & errors | Teardown: [`SharedState::request_shutdown`](../../crates/fpas-vm/src/vm/shared.rs), [`ShutdownAfterMain`](../../crates/fpas-vm/src/vm/mod.rs). Runtime failure: [`signal_runtime_failure`](../../crates/fpas-vm/src/vm/shared.rs). [`Worker::run`](../../crates/fpas-vm/src/vm/execute/mod.rs) checks **`abort_spawned_bytecode`** for spawned tasks; main uses `check_shutdown`. Pool join prefers main diagnostic if both fail. Tests: [`runtime/runtime_shutdown_happy.rs`](../../crates/fpas-vm/src/tests/runtime/runtime_shutdown_happy.rs), [`pool/pool_shutdown.rs`](../../crates/fpas-vm/src/tests/pool/pool_shutdown.rs). |

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

**TUI host bridge:** `TuiState` in the same module holds `TuiHost` and optional FP handler values; the [`fpas-vm` TUI execute path](../../crates/fpas-vm/src/vm/execute/io/tui/mod.rs) runs **`On*`-style** callbacks on the **main** worker only (via `Worker::call_function_sync`), without holding the `tui` mutex across the FP call. Hosted intrinsics **256**–**347** (including host widgets **343**–**347**) are summarized in [`docs/pascal/std/tui-app.md`](../pascal/std/tui-app.md). VM tests: [`core/tui_host_vm/`](../../crates/fpas-vm/src/tests/core/tui_host_vm/mod.rs) (host poll/register/process), [`core/tui_run_vm/`](../../crates/fpas-vm/src/tests/core/tui_run_vm/mod.rs) (`Application.Run` lifecycle).
