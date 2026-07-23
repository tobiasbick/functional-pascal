//! Direct VM coverage for typed scalar opcode families.

use fpas_bytecode::{Chunk, Op, Value};

use crate::tests::helpers::{emit_constant, loc, run_ok_output};

fn emit_binary(chunk: &mut Chunk, left: Value, right: Value, op: Op) {
    emit_constant(chunk, left);
    emit_constant(chunk, right);
    chunk.emit(op, loc());
    chunk.emit(Op::PrintLn, loc());
}

fn emit_unary(chunk: &mut Chunk, value: Value, op: Op) {
    emit_constant(chunk, value);
    chunk.emit(op, loc());
    chunk.emit(Op::PrintLn, loc());
}

#[test]
fn real_arithmetic_and_conversion_return_expected_values() {
    let mut chunk = Chunk::new();
    emit_binary(&mut chunk, Value::Real(2.5), Value::Real(1.5), Op::AddReal);
    emit_binary(&mut chunk, Value::Real(5.0), Value::Real(1.5), Op::SubReal);
    emit_binary(&mut chunk, Value::Real(2.0), Value::Real(3.0), Op::MulReal);
    emit_unary(&mut chunk, Value::Real(2.5), Op::NegateReal);
    emit_unary(&mut chunk, Value::Integer(3), Op::IntToReal);
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["4", "3.5", "6", "-2.5", "3"]);
}

#[test]
fn boolean_and_bitwise_operations_return_expected_values() {
    let mut chunk = Chunk::new();
    emit_binary(
        &mut chunk,
        Value::Boolean(true),
        Value::Boolean(false),
        Op::And,
    );
    emit_binary(
        &mut chunk,
        Value::Boolean(true),
        Value::Boolean(false),
        Op::Or,
    );
    emit_unary(&mut chunk, Value::Boolean(true), Op::Not);
    emit_binary(
        &mut chunk,
        Value::Boolean(true),
        Value::Boolean(true),
        Op::EqBool,
    );
    emit_binary(
        &mut chunk,
        Value::Boolean(true),
        Value::Boolean(false),
        Op::NeqBool,
    );
    emit_binary(&mut chunk, Value::Integer(6), Value::Integer(3), Op::BitAnd);
    emit_binary(&mut chunk, Value::Integer(6), Value::Integer(3), Op::BitOr);
    emit_binary(&mut chunk, Value::Integer(6), Value::Integer(3), Op::BitXor);
    emit_binary(&mut chunk, Value::Integer(6), Value::Integer(3), Op::Shl);
    emit_binary(&mut chunk, Value::Integer(8), Value::Integer(2), Op::Shr);
    chunk.emit(Op::Halt, loc());

    assert_eq!(
        run_ok_output(chunk),
        vec![
            "false", "true", "false", "true", "true", "2", "7", "5", "48", "2"
        ]
    );
}

#[test]
fn typed_comparisons_and_string_concat_return_expected_values() {
    let mut chunk = Chunk::new();
    emit_binary(&mut chunk, Value::Integer(3), Value::Integer(3), Op::EqInt);
    emit_binary(&mut chunk, Value::Integer(2), Value::Integer(3), Op::NeqInt);
    emit_binary(&mut chunk, Value::Integer(2), Value::Integer(3), Op::LtInt);
    emit_binary(&mut chunk, Value::Integer(3), Value::Integer(2), Op::GtInt);
    emit_binary(&mut chunk, Value::Integer(3), Value::Integer(3), Op::LeInt);
    emit_binary(&mut chunk, Value::Integer(3), Value::Integer(3), Op::GeInt);
    emit_binary(&mut chunk, Value::Real(1.0), Value::Real(2.0), Op::LtReal);
    emit_binary(&mut chunk, Value::Real(2.0), Value::Real(2.0), Op::EqReal);
    emit_binary(&mut chunk, Value::Real(2.0), Value::Real(3.0), Op::NeqReal);
    emit_binary(&mut chunk, Value::Real(3.0), Value::Real(2.0), Op::GtReal);
    emit_binary(&mut chunk, Value::Real(2.0), Value::Real(2.0), Op::LeReal);
    emit_binary(&mut chunk, Value::Real(2.0), Value::Real(2.0), Op::GeReal);
    emit_binary(
        &mut chunk,
        Value::Str("a".into()),
        Value::Str("a".into()),
        Op::EqStr,
    );
    emit_binary(
        &mut chunk,
        Value::Str("a".into()),
        Value::Str("b".into()),
        Op::NeqStr,
    );
    emit_binary(
        &mut chunk,
        Value::Str("a".into()),
        Value::Str("b".into()),
        Op::LtStr,
    );
    emit_binary(
        &mut chunk,
        Value::Str("b".into()),
        Value::Str("a".into()),
        Op::GtStr,
    );
    emit_binary(
        &mut chunk,
        Value::Str("a".into()),
        Value::Str("a".into()),
        Op::LeStr,
    );
    emit_binary(
        &mut chunk,
        Value::Str("a".into()),
        Value::Str("a".into()),
        Op::GeStr,
    );
    emit_binary(
        &mut chunk,
        Value::Str("functional ".into()),
        Value::Str("pascal".into()),
        Op::ConcatStr,
    );
    chunk.emit(Op::Halt, loc());

    assert_eq!(
        run_ok_output(chunk),
        vec![
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "functional pascal"
        ]
    );
}

#[test]
fn dynamic_arithmetic_and_ordering_return_expected_values() {
    let mut chunk = Chunk::new();
    emit_binary(&mut chunk, Value::Integer(2), Value::Integer(3), Op::AddDyn);
    emit_binary(&mut chunk, Value::Real(5.5), Value::Integer(2), Op::SubDyn);
    emit_binary(&mut chunk, Value::Integer(3), Value::Real(2.0), Op::MulDyn);
    emit_binary(&mut chunk, Value::Integer(3), Value::Real(2.0), Op::GtDyn);
    emit_binary(&mut chunk, Value::Integer(2), Value::Real(2.0), Op::LeDyn);
    emit_binary(&mut chunk, Value::Integer(2), Value::Real(2.0), Op::GeDyn);
    chunk.emit(Op::Halt, loc());

    assert_eq!(
        run_ok_output(chunk),
        vec!["5", "3.5", "6", "true", "true", "true"]
    );
}
