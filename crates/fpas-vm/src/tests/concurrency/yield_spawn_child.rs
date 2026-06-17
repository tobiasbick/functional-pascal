//! [`Op::Yield`] on spawned tasks (retained and detached) and main-task yields before `TaskWait`.
//!
//! **Documentation:** `docs/pascal/08-concurrency.md` (Phase 7), `docs/pascal/08-concurrency.md`

use fpas_bytecode::{Chunk, Intrinsic, Op, TaskIntrinsic, Value};

use crate::tests::helpers::{build_zero_arg_function_chunk, emit_constant, loc, run_ok_output};

// --- Positive: explicit `Yield` on spawned task ---

#[test]
fn retained_spawn_child_yield_then_return_still_waits_correctly() {
    let callee = "YieldThenSeven";
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
            chunk.emit(Op::Yield, loc());
            emit_constant(chunk, Value::Integer(7));
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["7"]);
}

#[test]
fn main_emits_many_yields_before_wait_child_still_completes() {
    let callee = "ReturnNine";
    let mut chunk = Chunk::new();
    emit_constant(
        &mut chunk,
        Value::Function {
            name: callee.to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());
    for _ in 0..64 {
        chunk.emit(Op::Yield, loc());
    }
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let code_start = chunk.len();
    chunk
        .functions
        .insert(callee.to_ascii_lowercase(), (code_start, 0));
    emit_constant(&mut chunk, Value::Integer(9));
    chunk.emit(Op::Return, loc());

    assert_eq!(run_ok_output(chunk), vec!["9"]);
}

#[test]
fn detached_spawn_child_yields_before_return() {
    let callee = "DetachedYield";
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
            chunk.emit(Op::SpawnDetachedTask(0), loc());
            emit_constant(chunk, Value::Integer(55));
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            chunk.emit(Op::Yield, loc());
            chunk.emit(Op::Unit, loc());
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["55"]);
}
