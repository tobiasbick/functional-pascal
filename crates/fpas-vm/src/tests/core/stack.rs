use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

use crate::tests::helpers::{emit_constant, loc, run_err};

/// Pushing STACK_MAX values must succeed; the next push must report a stack-overflow error.
#[test]
fn stack_overflow_is_reported_as_error_not_panic() {
    const STACK_MAX: usize = 4096;
    let mut chunk = Chunk::new();
    // Push STACK_MAX + 1 values — the last push must exceed the limit.
    for _ in 0..=STACK_MAX {
        emit_constant(&mut chunk, Value::Integer(0));
    }
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_INTRINSIC_STACK_STATE_ERROR);
    assert!(
        err.message.contains("Stack overflow"),
        "expected 'Stack overflow' in message, got: {}",
        err.message
    );
}
