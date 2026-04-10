use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::codes::{RUNTIME_DIVISION_BY_ZERO, RUNTIME_NUMERIC_DOMAIN_ERROR};

use super::helpers::{emit_constant, loc, run_err};

#[test]
fn integer_division_overflow_reports_error_instead_of_panicking() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(i64::MIN));
    emit_constant(&mut chunk, Value::Integer(-1));
    chunk.emit(Op::DivInt, loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
}

#[test]
fn integer_modulo_overflow_reports_error_instead_of_panicking() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(i64::MIN));
    emit_constant(&mut chunk, Value::Integer(-1));
    chunk.emit(Op::ModInt, loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
}

#[test]
fn integer_negation_overflow_reports_error_instead_of_panicking() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(i64::MIN));
    chunk.emit(Op::NegateInt, loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
}

#[test]
fn dynamic_negation_overflow_reports_error_instead_of_panicking() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(i64::MIN));
    chunk.emit(Op::NegateDyn, loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
}

#[test]
fn real_division_by_zero_reports_error_instead_of_returning_infinity() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Real(1.0));
    emit_constant(&mut chunk, Value::Real(0.0));
    chunk.emit(Op::DivReal, loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_DIVISION_BY_ZERO);
}

#[test]
fn dynamic_real_division_by_zero_reports_error() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Real(1.0));
    emit_constant(&mut chunk, Value::Real(0.0));
    chunk.emit(Op::DivDyn, loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_DIVISION_BY_ZERO);
}
