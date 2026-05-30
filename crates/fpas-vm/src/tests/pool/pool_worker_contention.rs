//! Two pool workers competing for a single queued task (exactly one runs the body).
//!
//! **Documentation:** `docs/rust/parallel-vm.md` (Phase 5 checklist), `docs/pascal/08-concurrency.md`

use crate::vm::{TaskState, Worker};
use fpas_bytecode::{Chunk, Op, Value};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::tests::helpers::{emit_constant, loc, minimal_shared_state};

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

    let start = std::time::Instant::now();
    while shared
        .console
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .output()
        .lines
        .is_empty()
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
