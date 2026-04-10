use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::codes::RUNTIME_UNDEFINED_GLOBAL;

use super::helpers::{emit_constant, loc, run_err, run_ok_output};

#[test]
fn set_global_then_get_global_round_trips_value() {
    let mut chunk = Chunk::new();
    let name_idx = chunk
        .add_constant(Value::Str("Answer".to_string()))
        .expect("constant should fit in test chunk");

    emit_constant(&mut chunk, Value::Integer(42));
    chunk.emit(Op::SetGlobal(name_idx), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::GetGlobal(name_idx), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["42"]);
}

#[test]
fn set_global_then_get_global_is_case_insensitive() {
    let mut chunk = Chunk::new();
    let set_name_idx = chunk
        .add_constant(Value::Str("Answer".to_string()))
        .expect("constant should fit in test chunk");
    let get_name_idx = chunk
        .add_constant(Value::Str("answer".to_string()))
        .expect("constant should fit in test chunk");

    emit_constant(&mut chunk, Value::Integer(42));
    chunk.emit(Op::SetGlobal(set_name_idx), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::GetGlobal(get_name_idx), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["42"]);
}

#[test]
fn get_global_on_missing_name_reports_runtime_error() {
    let mut chunk = Chunk::new();
    let name_idx = chunk
        .add_constant(Value::Str("Missing".to_string()))
        .expect("constant should fit in test chunk");
    chunk.emit(Op::GetGlobal(name_idx), loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_UNDEFINED_GLOBAL);
}
