//! Worker pool sizing when the chunk uses spawn opcodes.
//!
//! **Documentation:** `docs/rust/parallel-vm.md` (Phase 4 checklist), `docs/pascal/08-concurrency.md`

use crate::Vm;
use fpas_bytecode::{Chunk, Op};

use crate::tests::helpers::loc;

use super::worker_pool_common::expected_spawn_pool_workers;

// --- Positive: pool size policy ---

#[test]
fn no_spawn_bytecode_means_zero_worker_pool() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Halt, loc());
    assert!(!chunk.uses_spawn_tasks());
    let vm = Vm::new(chunk);
    assert_eq!(vm.worker_pool_size_for_tests(), 0);
}

#[test]
fn yield_only_chunk_does_not_start_worker_pool() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Yield, loc());
    chunk.emit(Op::Halt, loc());
    assert!(!chunk.uses_spawn_tasks());
    let vm = Vm::new(chunk);
    assert_eq!(vm.worker_pool_size_for_tests(), 0);
}

#[test]
fn spawn_task_chunk_uses_parallelism_policy() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::SpawnTask(0), loc());
    assert!(chunk.uses_spawn_tasks());
    let vm = Vm::new(chunk);
    assert_eq!(
        vm.worker_pool_size_for_tests(),
        expected_spawn_pool_workers(),
        "pool size must match max(1, available_parallelism - 1)"
    );
}

#[test]
fn spawn_detached_chunk_uses_same_pool_policy_as_retained_spawn() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::SpawnDetachedTask(0), loc());
    assert!(chunk.uses_spawn_tasks());
    let vm = Vm::new(chunk);
    assert_eq!(
        vm.worker_pool_size_for_tests(),
        expected_spawn_pool_workers()
    );
}

#[test]
fn spawn_pool_size_is_at_least_one_even_on_single_cpu_host() {
    // Policy: saturating_sub(1).max(1) — document edge case for 1 logical CPU.
    let mut chunk = Chunk::new();
    chunk.emit(Op::SpawnTask(0), loc());
    let vm = Vm::new(chunk);
    assert!(
        vm.worker_pool_size_for_tests() >= 1,
        "spawn programs must retain at least one pool worker"
    );
}

// --- Edge: cloning chunk does not change VM pool decision ---

#[test]
fn pool_size_follows_clone_scan_identically() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::SpawnTask(0), loc());
    let copy = chunk.clone();
    assert_eq!(chunk.uses_spawn_tasks(), copy.uses_spawn_tasks());
    let a = Vm::new(chunk).worker_pool_size_for_tests();
    let b = Vm::new(copy).worker_pool_size_for_tests();
    assert_eq!(a, b);
    assert_eq!(a, expected_spawn_pool_workers());
}
