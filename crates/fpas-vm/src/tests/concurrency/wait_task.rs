//! Blocking [`Std.Task.Wait`]: single-task wait, timeslice-heavy children, panic → shutdown.
//!
//! **Documentation:** `docs/pascal/08-concurrency.md` (Phase 8), `docs/pascal/std/task.md`, `docs/pascal/08-concurrency.md`

use fpas_bytecode::{Intrinsic, Op, TaskIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN;

use crate::tests::helpers::{
    build_zero_arg_function_chunk, emit_constant, loc, run_err, run_ok_output,
};

use super::fixtures::emit_instruction_waste;

// --- Positive: Wait completes under load; child runs on pool ---

#[test]
fn wait_succeeds_when_child_runs_long_enough_to_need_timeslice() {
    let callee = "BusyThenThree";
    let chunk = build_zero_arg_function_chunk(
        callee,
        |chunk| {
            emit_constant(
                chunk,
                Value::Function {
                    name: callee.to_string(),
                    captures: vec![],
                },
            );
            chunk.emit(Op::SpawnTask(0), loc());
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            emit_instruction_waste(chunk, 200);
            emit_constant(chunk, Value::Integer(3));
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["3"]);
}

#[test]
fn wait_then_second_spawn_and_wait_is_independent() {
    let f = "Inc";
    let chunk = build_zero_arg_function_chunk(
        f,
        |chunk| {
            for _ in 0..2 {
                emit_constant(
                    chunk,
                    Value::Function {
                        name: f.to_string(),
                        captures: vec![],
                    },
                );
                chunk.emit(Op::SpawnTask(0), loc());
                chunk.emit(
                    Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                    loc(),
                );
            }
            chunk.emit(Op::AddInt, loc());
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(10));
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["20"]);
}

// --- Negative ---

#[test]
fn wait_on_child_that_panics_surfaces_shutdown_to_waiter() {
    let callee = "Boom";
    let chunk = build_zero_arg_function_chunk(
        callee,
        |chunk| {
            emit_constant(
                chunk,
                Value::Function {
                    name: callee.to_string(),
                    captures: vec![],
                },
            );
            chunk.emit(Op::SpawnTask(0), loc());
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
        },
        |chunk| {
            emit_constant(chunk, Value::Str("x".into()));
            chunk.emit(Op::Panic, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_VM_SHUTDOWN);
}
