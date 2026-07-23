//! Direct VM coverage for conditional branches and non-newline output.

use fpas_bytecode::{Chunk, Op, Value};

use crate::tests::helpers::{emit_constant, loc, run_ok_output};

#[test]
fn conditional_jumps_select_the_expected_targets() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Boolean(false));
    chunk.emit(Op::JumpIfFalse(4), loc());
    emit_constant(&mut chunk, Value::Str("wrong".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Str("false branch".into()));
    chunk.emit(Op::PrintLn, loc());

    emit_constant(&mut chunk, Value::Boolean(true));
    chunk.emit(Op::JumpIfTrue(10), loc());
    emit_constant(&mut chunk, Value::Str("wrong".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Str("true branch".into()));
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["false branch", "true branch"]);
}

#[test]
fn print_without_newline_combines_with_following_println() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Str("functional ".into()));
    chunk.emit(Op::Print, loc());
    emit_constant(&mut chunk, Value::Str("pascal".into()));
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["functional pascal"]);
}
