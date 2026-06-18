//! Retained spawn: wait for child return value and multi-argument calls.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md` (Phase 6), `docs/pascal/language/concurrency/README.md`

use fpas_bytecode::{Intrinsic, Op, TaskIntrinsic, Value};

use crate::tests::helpers::{
    build_function_chunk, build_zero_arg_function_chunk, emit_constant, loc, run_ok_output,
};

#[test]
fn retained_spawn_wait_prints_child_return_value() {
    let callee = "ReturnFortyTwo";
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
            emit_constant(chunk, Value::Integer(42));
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["42"]);
}

#[test]
fn spawn_passes_two_arguments_and_child_returns_sum() {
    let callee = "AddTwo";
    let chunk = build_function_chunk(
        callee,
        2,
        |chunk| {
            emit_constant(chunk, Value::Integer(30));
            emit_constant(chunk, Value::Integer(12));
            emit_constant(
                chunk,
                Value::Function {
                    name: callee.to_string(),
                    captures: vec![],
                },
            );
            chunk.emit(Op::SpawnTask(2), loc());
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            chunk.emit(Op::GetLocal(0), loc());
            chunk.emit(Op::GetLocal(1), loc());
            chunk.emit(Op::AddInt, loc());
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["42"]);
}
