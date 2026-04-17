//! Worker pool sizing (conditional on spawn bytecode), scoped [`Vm::run`], shutdown waking pool waiters.
//!
//! **Documentation:** `docs/future/parallel-vm.md` (Phase 4 checklist), `docs/pascal/08-concurrency.md`

use crate::Vm;
use fpas_bytecode::{Chunk, Op, Value};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use super::helpers::{emit_constant, loc, minimal_shared_state};

/// Mirrors [`crate::vm::Vm::build`] pool sizing when the chunk uses spawn bytecode.
fn expected_spawn_pool_workers() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .saturating_sub(1)
        .max(1)
}

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

// --- Positive: scoped run completes and can be repeated ---

#[test]
fn run_with_zero_pool_workers_completes_halt_only_program() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Halt, loc());
    let mut vm = Vm::new(chunk);
    assert_eq!(vm.worker_pool_size_for_tests(), 0);
    vm.run().expect("halt-only run");
}

#[test]
fn repeated_vm_runs_with_zero_pool_each_complete() {
    for _ in 0..32 {
        let mut chunk = Chunk::new();
        chunk.emit(Op::Halt, loc());
        let mut vm = Vm::new(chunk);
        vm.run().expect("repeated short runs");
    }
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

// --- Positive: shutdown wakes every condvar waiter (empty queue) ---

#[test]
fn request_shutdown_unblocks_multiple_threads_waiting_on_task_available() {
    let mut c = Chunk::new();
    c.emit(Op::Halt, loc());
    let shared = Arc::new(minimal_shared_state(c));

    let n = 4;
    let entered = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    for _ in 0..n {
        let s = Arc::clone(&shared);
        let cnt = Arc::clone(&entered);
        handles.push(thread::spawn(move || {
            let mut guard = s.task_queue.lock().unwrap_or_else(|e| e.into_inner());
            cnt.fetch_add(1, Ordering::SeqCst);
            while !s.is_shutdown() {
                guard = s
                    .task_available
                    .wait(guard)
                    .unwrap_or_else(|e| e.into_inner());
            }
        }));
    }

    let start = std::time::Instant::now();
    while entered.load(Ordering::SeqCst) < n {
        assert!(
            start.elapsed() <= Duration::from_secs(2),
            "waiters did not all block on condvar in time"
        );
        thread::yield_now();
    }
    thread::sleep(Duration::from_millis(20));

    shared.request_shutdown();

    for h in handles {
        h.join()
            .expect("each waiter thread must exit after notify_all + shutdown flag");
    }
    assert!(shared.is_shutdown());
}

// --- Negative / edge: shutdown before waiters start still terminates pool_loop-style wait ---

#[test]
fn pool_worker_style_wait_exits_immediately_if_shutdown_already_set() {
    let mut c = Chunk::new();
    c.emit(Op::Halt, loc());
    let shared = Arc::new(minimal_shared_state(c));
    shared.request_shutdown();

    let s2 = Arc::clone(&shared);
    let handle = thread::spawn(move || {
        let mut guard = s2.task_queue.lock().unwrap_or_else(|e| e.into_inner());
        while !s2.is_shutdown() {
            guard = s2
                .task_available
                .wait(guard)
                .unwrap_or_else(|e| e.into_inner());
        }
    });

    handle.join().expect("waiter should not block");
}

// --- Positive: VM run joins pool after main (integration with real Vm) ---

#[test]
fn vm_run_with_spawn_chunk_returns_after_pool_workers_join() {
    let function_name = "Quick";
    let mut chunk = Chunk::new();
    emit_constant(
        &mut chunk,
        Value::Function {
            name: function_name.to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(Op::Halt, loc());

    let body_start = chunk.len();
    chunk
        .functions
        .insert(function_name.to_string(), (body_start, 0));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::Return, loc());

    assert!(chunk.uses_spawn_tasks());
    assert!(Vm::new(chunk.clone()).worker_pool_size_for_tests() > 0);

    let mut vm = Vm::new(chunk);
    vm.run().expect("main halts; pool workers idle then join");
}
