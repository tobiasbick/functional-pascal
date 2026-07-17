use fpas_bytecode::{Intrinsic, Op, Value};
use fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE;

use crate::tests::helpers::{build_function_chunk, emit_constant, loc, run_err};

#[test]
fn sync_call_rejects_halt_in_callback() {
    let callee = "Halts";
    let chunk = build_function_chunk(
        callee,
        1,
        |chunk| {
            emit_constant(chunk, Value::Array(vec![Value::Integer(1)]));
            emit_constant(
                chunk,
                Value::Function {
                    name: callee.to_string(),
                    captures: vec![],
                    task_bound: false,
                },
            );
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Array(
                    fpas_bytecode::ArrayIntrinsic::Map,
                ))),
                loc(),
            );
            chunk.emit(Op::Halt, loc());
        },
        |chunk| {
            chunk.emit(Op::Halt, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, INTERNAL_VM_INVARIANT_FAILURE);
    assert!(
        err.message
            .contains("Halt during synchronous function call"),
        "unexpected message: {}",
        err.message
    );
}
