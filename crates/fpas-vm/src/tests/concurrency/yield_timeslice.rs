//! Instruction timeslice scheduling: CPU-heavy spawned tasks and main-task ready-queue rules.
//!
//! **Documentation:** `docs/rust/parallel-vm.md` (Phase 7), `docs/pascal/08-concurrency.md`

use crate::vm::Worker;
use fpas_bytecode::{Chunk, Intrinsic, Op, Value};
use std::sync::Arc;

use crate::tests::helpers::{emit_constant, loc, minimal_shared_state, run_ok_output};

use super::fixtures::emit_instruction_waste;

// --- Positive: instruction timeslice lets two CPU-heavy spawned tasks finish ---

#[test]
fn two_wasteful_spawned_tasks_interleave_and_wait_all_completes() {
    // Each body runs far more than one timeslice of straight-line work so that, when only one
    // pool worker is available, the scheduler must suspend and re-queue each task repeatedly.
    // `WaitAll` proves both finished; two consecutive `Wait` ops would need extra stack handling.
    let slow_a = "SlowA";
    let slow_b = "SlowB";
    let mut chunk = Chunk::new();

    emit_constant(
        &mut chunk,
        Value::Function {
            name: slow_a.to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: slow_b.to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(Op::MakeArray(2), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TaskWaitAll as u16), loc());
    emit_constant(&mut chunk, Value::Str("both_done".to_string()));
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let fn_a_start = chunk.len();
    chunk
        .functions
        .insert(slow_a.to_ascii_lowercase(), (fn_a_start, 0));
    emit_instruction_waste(&mut chunk, 400);
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(Op::Return, loc());

    let fn_b_start = chunk.len();
    chunk
        .functions
        .insert(slow_b.to_ascii_lowercase(), (fn_b_start, 0));
    emit_instruction_waste(&mut chunk, 400);
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Return, loc());

    assert_eq!(run_ok_output(chunk), vec!["both_done"]);
}

// --- Edge / unit: main task never moves to the shared ready queue ---

#[test]
fn main_task_timeslice_does_not_enqueue_task_zero() {
    let mut c = Chunk::new();
    emit_instruction_waste(&mut c, 200);
    c.emit(Op::Halt, loc());
    let shared = Arc::new(minimal_shared_state(c));
    let mut main = Worker::new_main(Arc::clone(&shared));
    main.run().expect("main should finish");
    assert_eq!(main.current_task_id, 0);
    assert!(
        shared
            .task_queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .all(|t| t.id != 0),
        "task id 0 must never appear on the shared ready queue"
    );
}
