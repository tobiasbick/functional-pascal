//! Direct VM coverage for Result and Option construction, inspection, and unwrap opcodes.

use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::codes::RUNTIME_UNWRAP_FAILURE;

use crate::tests::helpers::{emit_constant, loc, run_err, run_ok_output};

#[test]
fn result_and_option_happy_paths_round_trip_payloads() {
    let mut chunk = Chunk::new();

    emit_constant(&mut chunk, Value::Integer(7));
    chunk.emit(Op::MakeOk, loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::IsResultOk, loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::UnwrapOk, loc());
    chunk.emit(Op::PrintLn, loc());

    emit_constant(&mut chunk, Value::Str("error".into()));
    chunk.emit(Op::MakeErr, loc());
    chunk.emit(Op::UnwrapErr, loc());
    chunk.emit(Op::PrintLn, loc());

    emit_constant(&mut chunk, Value::Integer(9));
    chunk.emit(Op::MakeSome, loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::IsOptionSome, loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::UnwrapSome, loc());
    chunk.emit(Op::PrintLn, loc());

    chunk.emit(Op::MakeNone, loc());
    chunk.emit(Op::IsOptionSome, loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(
        run_ok_output(chunk),
        vec!["true", "7", "error", "true", "9", "false"]
    );
}

#[test]
fn unwrap_some_rejects_none() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::MakeNone, loc());
    chunk.emit(Op::UnwrapSome, loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_UNWRAP_FAILURE);
}
