//! Detached [`Op::SpawnDetachedTask`]: stack effect (no retained handle).
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md` (Phase 6), `docs/pascal/language/concurrency/README.md`

use fpas_bytecode::{Op, Value};

use crate::tests::helpers::{build_zero_arg_function_chunk, emit_constant, loc, run_ok_output};

// --- Positive: detached spawn leaves no task handle ---

#[test]
fn detached_spawn_does_not_leave_task_handle_on_stack() {
    let callee = "SideEffectOnly";
    let chunk = build_zero_arg_function_chunk(
        callee,
        |chunk| {
            emit_constant(
                chunk,
                Value::Function {
                    name: callee.to_string(),
                    captures: vec![],
                    task_bound: false,
                },
            );
            chunk.emit(Op::SpawnDetachedTask(0), loc());
            emit_constant(chunk, Value::Integer(77));
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            chunk.emit(Op::Unit, loc());
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["77"]);
}
