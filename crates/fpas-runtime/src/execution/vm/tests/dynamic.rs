//! Dynamic arithmetic must retain checked numeric semantics and register lifetimes.

use std::sync::Arc;

use crate::execution::vm::value_ops::{self, BinaryOperation};
use crate::execution::vm::worker::Worker;
use crate::{Opcode, Value};

use super::{abc, return_unit, verified};

fn worker(opcode: Opcode, destination: u16) -> Worker {
    Worker::new(Arc::new(verified(
        vec![abc(opcode, destination, 0, 1), return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        3,
    )))
    .expect("dynamic worker")
}

#[test]
fn dynamic_integer_operations_preserve_checked_results_errors_and_aliases() {
    let values = [
        i64::MIN,
        i64::MIN + 1,
        -2,
        -1,
        0,
        1,
        2,
        i64::MAX - 1,
        i64::MAX,
    ];
    for (opcode, operation) in [
        (Opcode::AddDynamic, BinaryOperation::Add),
        (Opcode::SubtractDynamic, BinaryOperation::Subtract),
        (Opcode::MultiplyDynamic, BinaryOperation::Multiply),
        (Opcode::DivideDynamic, BinaryOperation::RealDivide),
        (Opcode::EqualDynamic, BinaryOperation::Equal),
        (Opcode::NotEqualDynamic, BinaryOperation::NotEqual),
        (Opcode::LessDynamic, BinaryOperation::Less),
        (Opcode::LessEqualDynamic, BinaryOperation::LessEqual),
        (Opcode::GreaterDynamic, BinaryOperation::Greater),
        (Opcode::GreaterEqualDynamic, BinaryOperation::GreaterEqual),
    ] {
        for destination in 0..3 {
            let mut worker = worker(opcode, destination);
            for left in values {
                for right in values {
                    worker.ip = 0;
                    worker.reset_registers(3);
                    worker
                        .store_register(0, Value::Integer(left))
                        .expect("left");
                    worker
                        .store_register(1, Value::Integer(right))
                        .expect("right");
                    let destination_index = usize::from(destination);
                    let previous = worker.registers[destination_index].clone();
                    let initialized = worker.register_is_initialized(destination_index);
                    let expected =
                        value_ops::binary(operation, &Value::Integer(left), &Value::Integer(right));
                    let actual = worker.dispatch_one();
                    match expected {
                        Ok(expected) => {
                            actual.expect("valid arithmetic");
                            let actual = &worker.registers[destination_index];
                            assert_eq!(actual, &expected, "{opcode:?}: {left}, {right}");
                            if let Value::Real(expected) = expected {
                                let Value::Real(actual) = actual else {
                                    panic!(
                                        "dynamic division must return real even for integer operands"
                                    );
                                };
                                assert_eq!(actual.to_bits(), expected.to_bits());
                            }
                            assert!(worker.register_is_initialized(destination_index));
                        }
                        Err(expected) => {
                            let error = actual.err().expect("invalid arithmetic must fail");
                            assert_eq!(error.code, expected.code);
                            assert_eq!(error.message, expected.message);
                            assert_eq!(error.help.as_deref(), Some(expected.hint.as_str()));
                            assert_eq!(worker.registers[destination_index], previous);
                            assert_eq!(
                                worker.register_is_initialized(destination_index),
                                initialized
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn dynamic_results_release_previous_destination_payloads() {
    for (opcode, expected) in [
        (Opcode::AddDynamic, Value::Integer(10)),
        (Opcode::GreaterDynamic, Value::Boolean(true)),
    ] {
        let mut worker = worker(opcode, 2);
        worker.store_register(0, Value::Integer(7)).expect("left");
        worker.store_register(1, Value::Integer(3)).expect("right");
        let cell = worker.domain.new_cell(Value::Unit).expect("old payload");
        worker
            .store_register(2, Value::Cell(cell))
            .expect("destination");
        assert_eq!(worker.domain.registered_count(), 1);
        worker.dispatch_one().expect("dynamic operation");
        assert_eq!(worker.domain.registered_count(), 0);
        assert_eq!(worker.registers[2], expected);
        assert!(worker.register_is_initialized(2));
        worker.ip = 0;
        worker.dispatch_one().expect("reuse scalar destination");
        assert_eq!(worker.registers[2], expected);
    }
}
