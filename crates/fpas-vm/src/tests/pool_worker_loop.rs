//! Pool worker pull loop, main vs pool task binding, [`TaskState`] save/load, and `enqueue_task` wakeups.
//!
//! **Documentation:** `docs/future/parallel-vm.md` (Phase 5 checklist), `docs/pascal/08-concurrency.md`

use crate::vm::{CallFrame, SharedState, TaskResultPoll, TaskState, Worker};
use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::DiagnosticStage;
use fpas_diagnostics::codes::{INTERNAL_VM_INVARIANT_FAILURE, RUNTIME_PROGRAM_PANIC};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use super::helpers::{emit_constant, loc, minimal_shared_state};

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
    assert_eq!(saved.retain_result, true);
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
    assert_eq!(w2.current_task_retain_result, true);
    assert_eq!(w2.stack.len(), 1);
    assert_eq!(w2.call_stack.len(), 1);
}

/// Bytecode for a spawned task starting at ip `0`: leave one return value and `Return`.
fn chunk_task_returns_integer(n: i64) -> Chunk {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(n));
    chunk.emit(Op::Return, loc());
    chunk
}

fn wait_for_task_result(shared: &SharedState, id: u64, timeout: Duration) -> Value {
    let start = Instant::now();
    loop {
        match shared.poll_task_result(id) {
            TaskResultPoll::Available(v) => return v,
            TaskResultPoll::Pending => {}
            TaskResultPoll::Consumed => panic!("task {id} result consumed before read"),
        }
        assert!(
            start.elapsed() < timeout,
            "timed out waiting for task {id} result"
        );
        thread::sleep(Duration::from_millis(2));
    }
}

// --- Positive: enqueue wakes a pool worker blocked on an empty queue ---

#[test]
fn enqueue_task_unblocks_pool_loop_waiting_on_condvar() {
    let chunk = chunk_task_returns_integer(41);
    let shared = Arc::new(minimal_shared_state(chunk));

    let s_loop = Arc::clone(&shared);
    let pool = thread::spawn(move || {
        let mut w = Worker::new_pool(s_loop);
        w.pool_loop()
    });

    thread::sleep(Duration::from_millis(40));

    shared.enqueue_task(TaskState {
        id: 1,
        ip: 0,
        stack: Vec::new(),
        call_stack: Vec::new(),
        retain_result: true,
    });

    let v = wait_for_task_result(&shared, 1, Duration::from_secs(2));
    assert_eq!(v, Value::Integer(41));

    shared.request_shutdown();
    pool.join()
        .expect("pool thread joins")
        .expect("pool_loop ok");
}

// --- Positive: fast dequeue path drains multiple tasks without extra waits ---

#[test]
fn pool_loop_drains_multiple_prequeued_tasks_before_blocking_again() {
    let chunk = chunk_task_returns_integer(2);
    let shared = Arc::new(minimal_shared_state(chunk));

    shared.enqueue_task(TaskState {
        id: 1,
        ip: 0,
        stack: Vec::new(),
        call_stack: Vec::new(),
        retain_result: true,
    });
    shared.enqueue_task(TaskState {
        id: 2,
        ip: 0,
        stack: Vec::new(),
        call_stack: Vec::new(),
        retain_result: true,
    });

    let s_loop = Arc::clone(&shared);
    let pool = thread::spawn(move || {
        let mut w = Worker::new_pool(s_loop);
        w.pool_loop()
    });

    assert_eq!(
        wait_for_task_result(&shared, 1, Duration::from_secs(2)),
        Value::Integer(2)
    );
    assert_eq!(
        wait_for_task_result(&shared, 2, Duration::from_secs(2)),
        Value::Integer(2)
    );

    shared.request_shutdown();
    pool.join().expect("join").expect("pool_loop ok");
}

// --- Edge: two pool workers, one task — exactly one runs the work ---

#[test]
fn two_pool_loops_share_one_task_only_one_executes_body() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Str("once".to_string()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));

    let s_a = Arc::clone(&shared);
    let a = thread::spawn(move || {
        let mut w = Worker::new_pool(s_a);
        w.pool_loop()
    });
    let s_b = Arc::clone(&shared);
    let b = thread::spawn(move || {
        let mut w = Worker::new_pool(s_b);
        w.pool_loop()
    });

    thread::sleep(Duration::from_millis(30));

    shared.enqueue_task(TaskState {
        id: 1,
        ip: 0,
        stack: Vec::new(),
        call_stack: Vec::new(),
        retain_result: false,
    });

    let start = Instant::now();
    while shared
        .console
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .output()
        .lines
        .len()
        < 1
    {
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "expected one printed line"
        );
        thread::sleep(Duration::from_millis(2));
    }

    let lines = shared
        .console
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .output()
        .lines
        .clone();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "once");

    shared.request_shutdown();
    a.join().expect("a").expect("a ok");
    b.join().expect("b").expect("b ok");
}

// --- Negative: illegal instruction pointer surfaces as error and sets shutdown ---

#[test]
fn pool_loop_task_with_ip_out_of_range_returns_error_and_shuts_down() {
    let chunk = chunk_task_returns_integer(0);
    let shared = Arc::new(minimal_shared_state(chunk));

    let s_loop = Arc::clone(&shared);
    let pool = thread::spawn(move || {
        let mut w = Worker::new_pool(s_loop);
        w.pool_loop()
    });

    shared.enqueue_task(TaskState {
        id: 9,
        ip: 10_000,
        stack: Vec::new(),
        call_stack: Vec::new(),
        retain_result: false,
    });

    let err = pool.join().expect("join").expect_err("vm error expected");
    assert_eq!(err.code, INTERNAL_VM_INVARIANT_FAILURE);
    assert_eq!(err.stage, DiagnosticStage::Internal);
    assert!(
        shared.is_shutdown(),
        "failed task must request global shutdown"
    );
}

#[test]
fn pool_loop_runtime_panic_opcode_requests_shutdown() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Str("boom".to_string()));
    chunk.emit(Op::Panic, loc());

    let shared = Arc::new(minimal_shared_state(chunk));

    let s_loop = Arc::clone(&shared);
    let pool = thread::spawn(move || {
        let mut w = Worker::new_pool(s_loop);
        w.pool_loop()
    });

    shared.enqueue_task(TaskState {
        id: 3,
        ip: 0,
        stack: Vec::new(),
        call_stack: Vec::new(),
        retain_result: false,
    });

    let err = pool.join().expect("join").expect_err("panic opcode");
    assert_eq!(err.code, RUNTIME_PROGRAM_PANIC);
    assert_eq!(err.stage, DiagnosticStage::Runtime);
    assert!(shared.is_shutdown());
}

// --- Edge: sentinel worker exits cleanly when shutdown is already set ---

#[test]
fn pool_loop_returns_immediately_when_shutdown_and_queue_empty() {
    let chunk = Chunk::new();
    let shared = Arc::new(minimal_shared_state(chunk));
    shared.request_shutdown();

    let mut w = Worker::new_pool(Arc::clone(&shared));
    w.pool_loop().expect("immediate exit");
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
