//! Typed real arithmetic preserves checked binary64 results, diagnostics, and storage lifetimes.
//!
//! **Documentation:** `docs/pascal/language/basics/operators.md`.

use std::sync::Arc;

use crate::execution::vm::value_ops::{self, BinaryOperation, ValueOperationError};
use crate::execution::vm::worker::Worker;
use crate::{Opcode, Value};

use super::{abc, return_unit, verified};

const OPERATIONS: [(Opcode, BinaryOperation); 4] = [
    (Opcode::AddReal, BinaryOperation::Add),
    (Opcode::SubtractReal, BinaryOperation::Subtract),
    (Opcode::MultiplyReal, BinaryOperation::Multiply),
    (Opcode::DivideReal, BinaryOperation::RealDivide),
];

fn worker(opcode: Opcode, destination: u16) -> Worker {
    Worker::new(Arc::new(verified(
        vec![abc(opcode, destination, 0, 1), return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        3,
    )))
    .expect("real arithmetic worker")
}

fn assert_same_value(actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Real(actual), Value::Real(expected)) => {
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
        _ => assert_eq!(actual, expected),
    }
}

fn check_case(
    worker: &mut Worker,
    sources: [Value; 2],
    destination: usize,
    initialize_destination: bool,
    expected: Result<Value, ValueOperationError>,
) {
    worker.ip = 0;
    worker.reset_registers(3);
    for (index, source) in sources.into_iter().enumerate() {
        worker.store_register(index, source).expect("source");
    }
    if initialize_destination {
        worker
            .store_register(destination, Value::Real(-13.25))
            .expect("previous destination");
    }
    let before = worker.registers.clone();
    let initialized: [bool; 3] = std::array::from_fn(|index| worker.register_is_initialized(index));
    let actual = worker.dispatch_one();
    match expected {
        Ok(expected) => {
            actual.expect("valid arithmetic");
            assert_same_value(&worker.registers[destination], &expected);
            assert!(worker.register_is_initialized(destination));
        }
        Err(expected) => {
            let expected = worker.runtime_error(expected.code, expected.message, expected.hint);
            assert_eq!(actual.err().expect("checked arithmetic failure"), expected);
            assert_same_value(&worker.registers[destination], &before[destination]);
            assert_eq!(
                worker.register_is_initialized(destination),
                initialized[destination]
            );
        }
    }
    for index in 0..3 {
        if index != destination {
            assert_same_value(&worker.registers[index], &before[index]);
            assert_eq!(worker.register_is_initialized(index), initialized[index]);
        }
    }
}

#[test]
fn typed_real_arithmetic_matches_checked_boundaries_bits_errors_and_aliases() {
    let values = [
        0.0,
        -0.0,
        f64::from_bits(1),
        -f64::from_bits(1),
        f64::from_bits(0x000f_ffff_ffff_ffff),
        -f64::from_bits(0x000f_ffff_ffff_ffff),
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
        0.5,
        -0.5,
        1.0,
        -1.0,
        2.0,
        -2.0,
        f64::MAX,
        -f64::MAX,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::from_bits(0x7ff8_0000_0000_0042),
        f64::from_bits(0x7ff0_0000_0000_0001),
        f64::from_bits(0xfff8_0000_0000_0042),
    ];
    for (opcode, operation) in OPERATIONS {
        for (destination, initialized) in [(0, false), (1, false), (2, false), (2, true)] {
            let mut worker = worker(opcode, destination);
            for left in values {
                for right in values {
                    check_case(
                        &mut worker,
                        [Value::Real(left), Value::Real(right)],
                        usize::from(destination),
                        initialized,
                        value_ops::real_binary(operation, left, right),
                    );
                }
            }
        }
    }
}

#[test]
fn typed_real_wrong_type_fallback_matches_generic_results_and_diagnostics() {
    let cases = [
        [Value::Integer(7), Value::Integer(3)],
        [Value::Integer(1), Value::Real(1.0)],
        [Value::Real(1.0), Value::Integer(1)],
        [Value::Str("a".into()), Value::Str("b".into())],
        [Value::Boolean(true), Value::Boolean(false)],
        [Value::Unit, Value::Real(1.0)],
        [Value::Real(1.0), Value::Str("x".into())],
    ];
    for (opcode, operation) in OPERATIONS {
        for (destination, initialized) in [(0, false), (1, false), (2, false), (2, true)] {
            let mut worker = worker(opcode, destination);
            for sources in &cases {
                let expected = value_ops::binary(operation, &sources[0], &sources[1]);
                check_case(
                    &mut worker,
                    sources.clone(),
                    usize::from(destination),
                    initialized,
                    expected,
                );
            }
        }
    }
}

#[test]
fn typed_real_results_release_cells_and_failures_keep_them() {
    for (opcode, operation) in OPERATIONS {
        for (left, right) in [(7.5, 2.0), (f64::INFINITY, 0.0)] {
            let mut worker = worker(opcode, 2);
            worker.store_register(0, Value::Real(left)).expect("left");
            worker.store_register(1, Value::Real(right)).expect("right");
            let cell = worker.domain.new_cell(Value::Unit).expect("owned cell");
            worker
                .store_register(2, Value::Cell(cell))
                .expect("destination");
            assert_eq!(worker.domain.registered_count(), 1);
            match value_ops::real_binary(operation, left, right) {
                Ok(expected) => {
                    worker.dispatch_one().expect("replace destination");
                    assert_same_value(&worker.registers[2], &expected);
                    assert_eq!(worker.domain.registered_count(), 0);
                    worker.ip = 0;
                    worker.dispatch_one().expect("reuse real destination");
                    assert_same_value(&worker.registers[2], &expected);
                }
                Err(expected) => {
                    let error = worker.dispatch_one().err().expect("invalid real input");
                    assert_eq!(
                        error,
                        worker.runtime_error(expected.code, expected.message, expected.hint)
                    );
                    assert!(matches!(worker.registers[2], Value::Cell(_)));
                    assert_eq!(worker.domain.registered_count(), 1);
                }
            }
            assert!(worker.register_is_initialized(2));
            assert_same_value(&worker.registers[0], &Value::Real(left));
            assert_same_value(&worker.registers[1], &Value::Real(right));
        }
    }
}
