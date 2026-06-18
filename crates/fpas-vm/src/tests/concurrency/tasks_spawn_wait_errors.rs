//! Spawn arity and invalid double-`Wait` on the same task handle.
//!
//! **Documentation:** `docs/pascal/std/concurrency/task.md`, `docs/pascal/language/concurrency/README.md`

use fpas_bytecode::{Intrinsic, Op, TaskIntrinsic, Value};
use fpas_diagnostics::codes::{RUNTIME_INVALID_TASK, RUNTIME_WRONG_CALL_ARITY};

use crate::tests::helpers::{
    build_function_chunk, build_zero_arg_function_chunk, emit_constant, loc, run_err,
};

#[test]
fn spawn_task_with_wrong_arity_reports_runtime_error() {
    let function_name = "NeedOneArg";
    let chunk = build_function_chunk(
        function_name,
        1,
        |chunk| {
            emit_constant(
                chunk,
                Value::Function {
                    name: function_name.to_string(),
                    captures: vec![],
                },
            );
            chunk.emit(Op::SpawnTask(0), loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(0));
            chunk.emit(Op::Return, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_WRONG_CALL_ARITY);
}

#[test]
fn waiting_twice_on_same_task_reports_runtime_error() {
    let function_name = "ReturnSeven";
    let chunk = build_zero_arg_function_chunk(
        function_name,
        |chunk| {
            emit_constant(
                chunk,
                Value::Function {
                    name: function_name.to_string(),
                    captures: vec![],
                },
            );
            chunk.emit(Op::SpawnTask(0), loc());
            chunk.emit(Op::Dup, loc());
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
            chunk.emit(Op::Pop, loc());
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(7));
            chunk.emit(Op::Return, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_INVALID_TASK);
}
