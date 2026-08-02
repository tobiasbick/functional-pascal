//! Blocking [`Std.Task.Wait`]: single-task wait, timeslice-heavy children, panic propagation.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md`,
//! `docs/pascal/std/concurrency/task.md`

use fpas_bytecode::{Chunk, Intrinsic, Op, TaskIntrinsic, Value};
use fpas_diagnostics::codes::{RUNTIME_INVALID_TASK, RUNTIME_PROGRAM_PANIC};

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
            emit_constant(chunk, Value::function(callee.to_string(), vec![], false));
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
                emit_constant(chunk, Value::function(f.to_string(), vec![], false));
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
fn wait_on_child_that_panics_surfaces_original_diagnostic() {
    let callee = "Boom";
    let chunk = build_zero_arg_function_chunk(
        callee,
        |chunk| {
            emit_constant(chunk, Value::function(callee.to_string(), vec![], false));
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
    assert_eq!(err.code, RUNTIME_PROGRAM_PANIC);
}

#[test]
fn wait_rejects_task_handle_not_created_by_vm() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Task(0));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_INVALID_TASK);
    assert!(err.message.contains("not created by this VM"));
}

#[test]
fn wait_rejects_forged_handle_for_detached_task() {
    let callee = "Detached";
    let chunk = build_zero_arg_function_chunk(
        callee,
        |chunk| {
            emit_constant(chunk, Value::function(callee.into(), vec![], false));
            chunk.emit(Op::SpawnDetachedTask(0), loc());
            emit_constant(chunk, Value::Task(1));
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
        },
        |chunk| {
            chunk.emit(Op::Unit, loc());
            chunk.emit(Op::Return, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_INVALID_TASK);
    assert!(err.message.contains("does not retain a result"));
}
