//! `WaitAll` validation and shutdown when a spawned child panics.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md` (Phase 8), `docs/pascal/std/concurrency/task.md`, `docs/pascal/language/concurrency/README.md`

use fpas_bytecode::{Chunk, Intrinsic, Op, TaskIntrinsic, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INVALID_TASK, RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_VM_SHUTDOWN,
};

use crate::tests::helpers::{emit_constant, loc, run_err};

// --- Negative ---

#[test]
fn wait_all_rejects_integer_element() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Integer(2));
    chunk.emit(Op::MakeArray(2), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::WaitAll))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
}

#[test]
fn wait_all_on_array_when_child_panicked_reports_shutdown() {
    let ok = "OkTask";
    let bad = "BadTask";
    let mut chunk = Chunk::new();

    emit_constant(
        &mut chunk,
        Value::Function {
            name: ok.to_string(),
            captures: vec![],
            task_bound: false,
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());

    emit_constant(
        &mut chunk,
        Value::Function {
            name: bad.to_string(),
            captures: vec![],
            task_bound: false,
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());

    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::MakeArray(2), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::WaitAll))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let code_start = chunk.len();
    chunk.insert_function(ok.to_ascii_lowercase(), code_start, 0);
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::Return, loc());

    let code_start_bad = chunk.len();
    chunk.insert_function(bad.to_ascii_lowercase(), code_start_bad, 0);
    emit_constant(&mut chunk, Value::Str("e".into()));
    chunk.emit(Op::Panic, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_VM_SHUTDOWN);
}

#[test]
fn wait_all_rejects_unknown_task_handle() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Task(999));
    chunk.emit(Op::MakeArray(1), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::WaitAll))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_INVALID_TASK);
    assert!(err.message.contains("Task 999"));
}
