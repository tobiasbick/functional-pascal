//! Direct VM coverage for local mutation, enclosing locals, cells, and local arrays.

use fpas_bytecode::{Chunk, Op, Value};

use crate::tests::helpers::{emit_constant, loc, run_ok_output};

#[test]
fn set_local_replaces_the_selected_stack_slot() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Integer(2));
    chunk.emit(Op::SetLocal(0), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["2"]);
}

#[test]
fn set_enclosing_updates_the_callers_local() {
    let mut chunk = Chunk::new();
    let function_name = chunk
        .add_constant(Value::Str("update".into()))
        .expect("function name");
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::Call(function_name, 0), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let function_start = chunk.len();
    chunk.insert_function("update", function_start, 0);
    emit_constant(&mut chunk, Value::Integer(2));
    chunk.emit(Op::SetEnclosing(1, 0), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::Unit, loc());
    chunk.emit(Op::Return, loc());

    assert_eq!(run_ok_output(chunk), vec!["2"]);
}

#[test]
fn cell_set_updates_the_shared_capture_value() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::MakeCell, loc());
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(2));
    chunk.emit(Op::CellSet, loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::CellGet, loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["2"]);
}

#[test]
fn local_array_push_and_pop_round_trip_the_new_tail() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Array(vec![Value::Integer(1)].into()));
    emit_constant(&mut chunk, Value::Integer(2));
    chunk.emit(Op::ArrayPushLocal(0, 0), loc());
    chunk.emit(Op::ArrayPopLocal(0, 0), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["2"]);
}
