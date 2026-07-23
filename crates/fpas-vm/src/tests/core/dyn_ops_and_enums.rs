//! Dynamic equality/ordering and typed `IsVariant` checks.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`,
//! `docs/pascal/language/types/enums.md`

use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

use crate::tests::helpers::{emit_constant, loc, run_err, run_ok_output};

#[test]
fn eq_dyn_compares_arrays_structurally() {
    let mut chunk = Chunk::new();
    emit_constant(
        &mut chunk,
        Value::Array(vec![Value::Integer(1), Value::Integer(2)].into()),
    );
    emit_constant(
        &mut chunk,
        Value::Array(vec![Value::Integer(1), Value::Integer(2)].into()),
    );
    chunk.emit(Op::EqDyn, loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["true"]);
}

#[test]
fn neq_dyn_compares_option_structurally() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::OptionSome(Box::new(Value::Integer(1))));
    emit_constant(&mut chunk, Value::OptionSome(Box::new(Value::Integer(2))));
    chunk.emit(Op::NeqDyn, loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["true"]);
}

#[test]
fn lt_dyn_rejects_option_operands() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::OptionSome(Box::new(Value::Integer(1))));
    emit_constant(&mut chunk, Value::OptionSome(Box::new(Value::Integer(2))));
    chunk.emit(Op::LtDyn, loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
}

#[test]
fn is_variant_requires_matching_type_and_variant() {
    let mut chunk = Chunk::new();
    let type_a = chunk
        .add_constant(Value::Str("Color".into()))
        .expect("constant");
    let type_b = chunk
        .add_constant(Value::Str("Light".into()))
        .expect("constant");
    let red = chunk
        .add_constant(Value::Str("Red".into()))
        .expect("constant");

    chunk.emit(Op::MakeEnum(type_a, red, 0), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::IsVariant(type_a, red), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::IsVariant(type_b, red), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["true", "false"]);
}
