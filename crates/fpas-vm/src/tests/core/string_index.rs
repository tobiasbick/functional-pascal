use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::codes::RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS;

use crate::tests::helpers::{emit_constant, loc, run_err};

#[test]
fn string_index_rejects_out_of_bounds_without_scanning_whole_string() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Str("hi".into()));
    emit_constant(&mut chunk, Value::Integer(1_000));
    chunk.emit(Op::IndexGet, loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);
}
