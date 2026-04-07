# Parallel stack VM

This note describes how the **`fpas-vm` crate** runs FPAS bytecode when programs use **`go`** tasks. For the language rules (`go`, `task`, `Wait`, `WaitAll`), see [`docs/pascal/08-concurrency.md`](../pascal/08-concurrency.md).

## Execution model

- **Main task (id 0)** runs on the thread that calls [`Vm::run`](../../crates/fpas-vm/src/vm/mod.rs): the main program’s bytecode executes there until `Op::Halt` or completion.
- **Spawned tasks** are scheduled on **worker threads** that pull work from a shared **ready queue** (`SharedState::task_queue`), coordinated with a **condition variable** (`task_available`).
- Bytecode uses **`Op::SpawnTask`** / **`Op::SpawnDetachedTask`** to enqueue new tasks; **`Op::Yield`** cooperates with scheduling. Waiting is implemented via intrinsics such as **`Std.Task.Wait`** / **`WaitAll`** (`Intrinsic::TaskWait`, `TaskWaitAll`).

## Thread pool

When [`Vm`](../../crates/fpas-vm/src/vm/mod.rs) is built:

- If the compiled [`Chunk`](../../crates/fpas-bytecode/src/chunk.rs) **does not** contain any spawn opcodes (`SpawnTask`, `SpawnDetachedTask`), **no** background worker threads are started. The main task runs alone; this keeps tests and single-threaded programs cheap when many VMs run in parallel.
- If the chunk **does** use spawn opcodes, the VM starts a pool whose size is **`max(1, available_parallelism − 1)`** (see `Vm::build` in `crates/fpas-vm/src/vm/mod.rs`). Workers block on the task queue until shutdown.

So the pool is tied to **whether the compiler emitted spawn instructions**, not to a heuristic at runtime.

## Shared state

[`SharedState`](../../crates/fpas-vm/src/vm/shared.rs) holds the immutable **`Chunk`**, the task queue and task results, **`shutdown`**, and mutex-protected **console**, **text/key input**, and **`Std.Tui`** session state. All workers share this `Arc` and synchronize per field as documented in the Rust sources.

## Related code

| Area | Location |
|------|----------|
| VM entry, scoped threads, pool size | `crates/fpas-vm/src/vm/mod.rs` |
| Worker loop, main vs pool | `crates/fpas-vm/src/vm/worker.rs` |
| Spawn / yield / wait | `crates/fpas-vm/src/vm/execute/concurrency/` |
| Chunk spawn detection | `Chunk::uses_spawn_tasks` in `crates/fpas-bytecode/src/chunk.rs` |
