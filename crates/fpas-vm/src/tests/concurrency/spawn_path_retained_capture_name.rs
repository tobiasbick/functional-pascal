//! Retained spawn: closure captures on child stack and canonical function-name lookup.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md` (Phase 6), `docs/pascal/language/concurrency/README.md`

use fpas_bytecode::{Chunk, Intrinsic, Op, TaskIntrinsic, Value};

use crate::tests::helpers::{build_zero_arg_function_chunk, emit_constant, loc, run_ok_output};

#[test]
fn spawn_loads_closure_captures_onto_child_stack() {
    let callee = "ReturnCapture";
    let chunk = build_zero_arg_function_chunk(
        callee,
        |chunk| {
            emit_constant(
                chunk,
                Value::function(callee.to_string(), vec![Value::Integer(99)], false),
            );
            chunk.emit(Op::SpawnTask(0), loc());
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            chunk.emit(Op::GetLocal(0), loc());
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["99"]);
}

#[test]
fn spawn_resolves_function_via_canonical_name_key() {
    // Chunk registers the callee under the lowercased key; the function *value* keeps mixed case.
    // Second lookup in spawn uses `canonical_name` of the value's name (see `exec_spawn_task`).
    let mut chunk = Chunk::new();
    emit_constant(
        &mut chunk,
        Value::function("MixedCaseFn".to_string(), vec![], false),
    );
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let code_start = chunk.len();
    chunk.insert_function("mixedcasefn".to_string(), code_start, 0);
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::Return, loc());

    assert_eq!(run_ok_output(chunk), vec!["1"]);
}
