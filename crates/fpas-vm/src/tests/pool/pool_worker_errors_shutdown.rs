//! Pool loop failures (bad IP, `Panic`), clean exit when shutdown is already set.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md` (Phase 5 checklist), `docs/pascal/language/concurrency/README.md`

use crate::vm::{TaskState, Worker};
use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::DiagnosticStage;
use fpas_diagnostics::codes::{INTERNAL_VM_INVARIANT_FAILURE, RUNTIME_PROGRAM_PANIC};
use std::sync::Arc;
use std::thread;

use crate::tests::helpers::{emit_constant, loc, minimal_shared_state};

use super::pool_worker_common::chunk_task_returns_integer;

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
    emit_constant(&mut chunk, Value::Str(("boom".to_string()).into()));
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
