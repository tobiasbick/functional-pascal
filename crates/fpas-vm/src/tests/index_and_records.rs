use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::codes::{
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS, RUNTIME_DICT_KEY_NOT_FOUND, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::helpers::{emit_constant, loc, run_err, run_ok_output};

#[test]
fn string_index_get_returns_character_value() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Str("pascal".to_string()));
    emit_constant(&mut chunk, Value::Integer(2));
    chunk.emit(Op::IndexGet, loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["s"]);
}

#[test]
fn array_index_get_with_negative_index_reports_bounds_error() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(7));
    chunk.emit(Op::MakeArray(1), loc());
    emit_constant(&mut chunk, Value::Integer(-1));
    chunk.emit(Op::IndexGet, loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);
}

#[test]
fn dict_index_set_adds_new_key_and_makes_it_readable() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::MakeDict(0), loc());
    emit_constant(&mut chunk, Value::Str("language".to_string()));
    emit_constant(&mut chunk, Value::Integer(2024));
    chunk.emit(Op::IndexSet, loc());
    emit_constant(&mut chunk, Value::Str("language".to_string()));
    chunk.emit(Op::IndexGet, loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["2024"]);
}

#[test]
fn dict_index_get_missing_key_reports_runtime_error() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Str("key".to_string()));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::MakeDict(1), loc());
    emit_constant(&mut chunk, Value::Str("other".to_string()));
    chunk.emit(Op::IndexGet, loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_DICT_KEY_NOT_FOUND);
}

#[test]
fn update_record_overrides_selected_field_and_keeps_others() {
    let mut chunk = Chunk::new();
    let type_idx = chunk
        .add_constant(Value::Str("Point".to_string()))
        .expect("constant should fit in test chunk");
    let x_idx = chunk
        .add_constant(Value::Str("x".to_string()))
        .expect("constant should fit in test chunk");
    let y_idx = chunk
        .add_constant(Value::Str("y".to_string()))
        .expect("constant should fit in test chunk");

    emit_constant(&mut chunk, Value::Str("x".to_string()));
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Str("y".to_string()));
    emit_constant(&mut chunk, Value::Integer(2));
    chunk.emit(Op::MakeRecord(type_idx, 2), loc());
    emit_constant(&mut chunk, Value::Str("x".to_string()));
    emit_constant(&mut chunk, Value::Integer(9));
    chunk.emit(Op::UpdateRecord(1), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::FieldGet(x_idx), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::FieldGet(y_idx), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["9", "2"]);
}

#[test]
fn update_record_with_unknown_field_reports_runtime_error() {
    let mut chunk = Chunk::new();
    let type_idx = chunk
        .add_constant(Value::Str("Point".to_string()))
        .expect("constant should fit in test chunk");

    emit_constant(&mut chunk, Value::Str("x".to_string()));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::MakeRecord(type_idx, 1), loc());
    emit_constant(&mut chunk, Value::Str("y".to_string()));
    emit_constant(&mut chunk, Value::Integer(2));
    chunk.emit(Op::UpdateRecord(1), loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
}
