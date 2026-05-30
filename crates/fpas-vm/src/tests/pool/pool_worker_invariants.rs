//! Pool worker constructors, [`TaskState`] save/load, timeslice reset after `load_task`.
//!
//! **Documentation:** `docs/rust/parallel-vm.md` (Phase 5 checklist), `docs/pascal/08-concurrency.md`

use crate::vm::{CallFrame, TaskState, Worker};
use fpas_bytecode::{Chunk, Value};
use std::sync::Arc;

use crate::tests::helpers::minimal_shared_state;

// --- Positive: constructors match main vs pool invariants ---

#[test]
fn main_worker_binds_task_zero_at_program_entry() {
    let chunk = Chunk::new();
    let shared = Arc::new(minimal_shared_state(chunk));
    let w = Worker::new_main(Arc::clone(&shared));
    assert_eq!(w.current_task_id, 0);
    assert_eq!(w.ip, 0);
    assert!(!w.current_task_retain_result);
}

#[test]
fn pool_worker_starts_with_sentinel_no_task_loaded() {
    let chunk = Chunk::new();
    let shared = Arc::new(minimal_shared_state(chunk));
    let w = Worker::new_pool(Arc::clone(&shared));
    assert_eq!(w.current_task_id, u64::MAX);
    assert_eq!(w.ip, 0);
}

// --- Positive: save/load migrates full task state ---

#[test]
fn save_task_and_load_task_round_trip_preserves_execution_state() {
    let chunk = Chunk::new();
    let shared = Arc::new(minimal_shared_state(chunk));
    let mut w = Worker::new_main(shared);
    w.ip = 12;
    w.current_task_id = 7;
    w.current_task_retain_result = true;
    w.stack.push(Value::Str("x".to_string()));
    w.call_stack.push(CallFrame {
        return_ip: 3,
        base_slot: 1,
    });

    let saved = w.save_task();
    assert_eq!(saved.id, 7);
    assert_eq!(saved.ip, 12);
    assert!(saved.retain_result);
    assert_eq!(saved.stack.len(), 1);
    assert_eq!(saved.call_stack.len(), 1);
    assert!(
        w.stack.is_empty(),
        "save_task must take stacks via mem::take"
    );
    assert!(
        w.call_stack.is_empty(),
        "save_task must take call stacks via mem::take"
    );

    let mut w2 = Worker::new_pool(Arc::clone(&w.shared));
    w2.load_task(saved);
    assert_eq!(w2.current_task_id, 7);
    assert_eq!(w2.ip, 12);
    assert!(w2.current_task_retain_result);
    assert_eq!(w2.stack.len(), 1);
    assert_eq!(w2.call_stack.len(), 1);
}

// --- Edge: load_task resets instruction budget for fair scheduling ---

#[test]
fn load_task_resets_timeslice_counter() {
    let chunk = Chunk::new();
    let shared = Arc::new(minimal_shared_state(chunk));
    let mut w = Worker::new_pool(Arc::clone(&shared));
    w.instructions_until_yield = 1;
    w.load_task(TaskState {
        id: 4,
        ip: 0,
        stack: Vec::new(),
        call_stack: Vec::new(),
        retain_result: false,
    });
    // Keep in sync with `TIMESLICE` in `crates/fpas-vm/src/vm/mod.rs`.
    assert_eq!(w.instructions_until_yield, 256);
}
