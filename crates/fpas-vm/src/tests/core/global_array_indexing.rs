//! Regression coverage for direct indexed writes to unit globals.

use fpas_bytecode::{Chunk, Op, Value};

use crate::tests::helpers::{emit_constant, loc, run_ok_output};

fn nested_array() -> Value {
    Value::Array(
        vec![
            Value::Array(vec![Value::Integer(1), Value::Integer(2)].into()),
            Value::Array(vec![Value::Integer(3), Value::Integer(4)].into()),
        ]
        .into(),
    )
}

#[test]
fn global_index_set_updates_a_nested_array() {
    let mut chunk = Chunk::new();
    let name_idx = chunk
        .add_constant(Value::Str("Grid".to_owned()))
        .expect("constant should fit in test chunk");

    emit_constant(&mut chunk, nested_array());
    chunk.emit(Op::SetGlobal(name_idx), loc());
    chunk.emit(Op::Pop, loc());
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(9));
    chunk.emit(Op::GlobalIndexSet(name_idx, 2), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::GetGlobal(name_idx), loc());
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::IndexGet, loc());
    emit_constant(&mut chunk, Value::Integer(0));
    chunk.emit(Op::IndexGet, loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["9"]);
}

#[test]
fn global_index_set_preserves_value_copy_semantics() {
    let mut chunk = Chunk::new();
    let name_idx = chunk
        .add_constant(Value::Str("Numbers".to_owned()))
        .expect("constant should fit in test chunk");

    emit_constant(
        &mut chunk,
        Value::Array(vec![Value::Integer(1), Value::Integer(2)].into()),
    );
    chunk.emit(Op::SetGlobal(name_idx), loc());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(9));
    chunk.emit(Op::GlobalIndexSet(name_idx, 1), loc());
    chunk.emit(Op::Pop, loc());
    emit_constant(&mut chunk, Value::Integer(0));
    chunk.emit(Op::IndexGet, loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::GetGlobal(name_idx), loc());
    emit_constant(&mut chunk, Value::Integer(0));
    chunk.emit(Op::IndexGet, loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["1", "9"]);
}
