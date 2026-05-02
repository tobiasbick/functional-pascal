use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::codes::{RUNTIME_DIVISION_BY_ZERO, RUNTIME_NUMERIC_DOMAIN_ERROR};

use crate::tests::helpers::{emit_constant, loc, run_err};

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

/// AddInt / SubInt / MulInt intentionally wrap on overflow (no panic, no error).
#[test]
fn add_int_wraps_on_overflow() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(i64::MAX));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::AddInt, loc());
    chunk.emit(Op::Halt, loc());

    // wrapping_add: i64::MAX + 1 == i64::MIN
    let output = crate::tests::helpers::run_ok_output(chunk);
    let _ = output; // result is on the stack but not printed; the test just confirms no panic/error
}

#[test]
fn sub_int_wraps_on_underflow() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(i64::MIN));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::SubInt, loc());
    chunk.emit(Op::Halt, loc());

    let output = crate::tests::helpers::run_ok_output(chunk);
    let _ = output;
}

#[test]
fn mul_int_wraps_on_overflow() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(i64::MAX));
    emit_constant(&mut chunk, Value::Integer(2));
    chunk.emit(Op::MulInt, loc());
    chunk.emit(Op::Halt, loc());

    let output = crate::tests::helpers::run_ok_output(chunk);
    let _ = output;
}
