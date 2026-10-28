//! Borrowed aggregate reads preserve aliases, diagnostics, and value lifetimes.
//!
//! **Documentation:** `docs/pascal/language/basics/operators.md`.

use std::sync::Arc;

use crate::execution::vm::value_ops::{self, BinaryOperation, ValueOperationError};
use crate::execution::vm::worker::Worker;
use crate::{Opcode, Value};

use super::{abc, return_unit, verified};

fn worker(opcode: Opcode, destination: u16, left: u16, right: u16) -> Worker {
    Worker::new(Arc::new(verified(
        vec![abc(opcode, destination, left, right), return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        3,
    )))
    .expect("aggregate reader")
}

fn array(values: Vec<Value>) -> Value {
    Value::Array(values.into())
}

fn key(number: i64) -> Value {
    Value::Record(crate::SharedRecord::new(
        Arc::new(crate::RuntimeRecordLayout {
            record: crate::RecordTypeId::new(0),
            type_name: "Key".to_owned(),
            fields: vec!["item".to_owned()],
        }),
        vec![Value::option_some(Value::Integer(number))],
    ))
}

fn checked(opcode: Opcode, left: &Value, right: &Value) -> Result<Value, ValueOperationError> {
    match opcode {
        Opcode::IndexGet => value_ops::index(left, right),
        Opcode::Contains => value_ops::binary(BinaryOperation::In, left, right),
        _ => unreachable!("aggregate read fixture"),
    }
}

fn check(opcode: Opcode, destination: u16, left: u16, right: u16, inputs: [Option<Value>; 3]) {
    let mut worker = worker(opcode, destination, left, right);
    for (index, input) in inputs.into_iter().enumerate() {
        if let Some(value) = input {
            worker.store_register(index, value).expect("input");
        }
    }
    let expected = checked(
        opcode,
        &worker.registers[usize::from(left)],
        &worker.registers[usize::from(right)],
    );
    let before = worker.registers.clone();
    let initialized: [bool; 3] = std::array::from_fn(|index| worker.register_is_initialized(index));
    let actual = worker.dispatch_one();
    let destination = usize::from(destination);
    match expected {
        Ok(expected) => {
            actual.expect("valid aggregate read");
            assert_eq!(worker.registers[destination], expected);
            assert!(worker.register_is_initialized(destination));
        }
        Err(expected) => {
            assert_eq!(
                actual.err().expect("invalid aggregate read"),
                worker.runtime_error(expected.code, expected.message, expected.hint)
            );
            assert_eq!(worker.registers[destination], before[destination]);
            assert_eq!(
                worker.register_is_initialized(destination),
                initialized[destination]
            );
        }
    }
    for index in 0..3 {
        if index != destination {
            assert_eq!(worker.registers[index], before[index]);
            assert_eq!(worker.register_is_initialized(index), initialized[index]);
        }
    }
}

fn check_aliases(opcode: Opcode, cases: Vec<[Value; 2]>) {
    for sources in cases {
        for (destination, initialized) in [(0, false), (1, false), (2, false), (2, true)] {
            check(
                opcode,
                destination,
                0,
                1,
                [
                    Some(sources[0].clone()),
                    Some(sources[1].clone()),
                    initialized.then_some(Value::Integer(-13)),
                ],
            );
        }
    }
}

#[test]
fn index_get_preserves_results_errors_and_destination_aliases() {
    let mut cases = Vec::new();
    for collection in [
        array(vec![Value::Integer(7), array(vec![Value::Integer(8)])]),
        array(Vec::new()),
        Value::Str("aé𝄞z".into()),
        Value::Str("".into()),
    ] {
        for index in [-1, 0, 1, 2, 3, 4, i64::MAX] {
            cases.push([collection.clone(), Value::Integer(index)]);
        }
        for index in [
            Value::Boolean(false),
            Value::Real(0.0),
            Value::Str("0".into()),
        ] {
            cases.push([collection.clone(), index]);
        }
    }
    let dictionary = Value::dict(vec![
        (key(7), Value::result_ok(Value::Str("result".into()))),
        (key(8), Value::Boolean(false)),
    ]);
    for index in [key(7), key(8), key(9), Value::Str("item".into())] {
        cases.push([dictionary.clone(), index]);
    }
    cases.extend([
        [Value::dict(Vec::new()), key(7)],
        [Value::Integer(7), Value::Integer(0)],
        [Value::OptionNone, Value::Integer(0)],
    ]);
    check_aliases(Opcode::IndexGet, cases);
}

#[test]
fn contains_preserves_results_errors_and_destination_aliases() {
    let mut cases = vec![
        [
            Value::Integer(7),
            array(vec![Value::Real(7.0), Value::Str("7".into())]),
        ],
        [Value::Integer(7), array(vec![Value::Integer(7)])],
        [Value::Integer(7), array(Vec::new())],
        [
            Value::option_some(key(7)),
            array(vec![Value::option_some(key(7))]),
        ],
        [Value::Integer(7), Value::Str("7".into())],
        [Value::Unit, Value::Integer(7)],
    ];
    let dictionary = Value::dict(vec![(key(7), Value::Str("result".into()))]);
    for needle in [key(7), key(9), Value::Str("result".into())] {
        cases.push([needle.clone(), array(vec![key(7), key(8)])]);
        cases.push([needle, dictionary.clone()]);
    }
    for text in ["", "aé𝄞z", "e\u{301}"] {
        for needle in ["", "a", "é", "𝄞", "é𝄞", "x", "\u{301}"] {
            cases.push([Value::Str(needle.into()), Value::Str(text.into())]);
        }
    }
    check_aliases(Opcode::Contains, cases);
}

#[test]
fn aggregate_reads_preserve_shared_operands_and_uninitialized_slots() {
    for opcode in [Opcode::IndexGet, Opcode::Contains] {
        for source in [
            Value::Str("same".into()),
            array(Vec::new()),
            Value::Integer(7),
        ] {
            for destination in [0, 2] {
                check(
                    opcode,
                    destination,
                    0,
                    0,
                    [Some(source.clone()), None, None],
                );
            }
        }
        for destination in [0, 1, 2] {
            check(
                opcode,
                destination,
                0,
                1,
                [None, Some(array(vec![Value::Unit])), None],
            );
            check(
                opcode,
                destination,
                0,
                1,
                [Some(array(Vec::new())), None, None],
            );
            check(
                opcode,
                destination,
                0,
                1,
                [None, None, Some(Value::Integer(-13))],
            );
        }
    }
}

#[test]
fn index_get_keeps_selected_cells_alive_when_overwriting_the_only_parent() {
    for dictionary in [false, true] {
        let mut worker = worker(Opcode::IndexGet, 0, 0, 1);
        let kept = worker
            .domain
            .new_cell(Value::Integer(42))
            .expect("selected cell");
        let discarded = worker.domain.new_cell(Value::Unit).expect("other cell");
        let kept_weak = kept.downgrade();
        let discarded_weak = discarded.downgrade();
        let selected = Value::option_some(Value::Cell(kept));
        let other = Value::option_some(Value::Cell(discarded));
        let parent = if dictionary {
            Value::dict(vec![
                (Value::Integer(0), selected),
                (Value::Integer(1), other),
            ])
        } else {
            array(vec![selected, other])
        };
        worker.store_register(0, parent).expect("only parent");
        worker.store_register(1, Value::Integer(0)).expect("index");
        assert_eq!(worker.domain.registered_count(), 2);
        worker
            .dispatch_one()
            .expect("read before overwriting parent");
        assert_eq!(worker.domain.registered_count(), 1);
        assert!(discarded_weak.upgrade().is_none());
        let Value::OptionSome(payload) = &worker.registers[0] else {
            panic!("selected payload");
        };
        let Value::Cell(cell) = &**payload else {
            panic!("selected cell");
        };
        assert_eq!(
            cell.read(&worker.domain).expect("live selected value"),
            Value::Integer(42)
        );
        worker
            .store_register(0, Value::Unit)
            .expect("release selected payload");
        // Direct register mutation must also refresh the roots published by dispatch.
        worker
            .publish_execution(&[])
            .expect("publish released register roots");
        assert_eq!(worker.domain.registered_count(), 0);
        assert!(kept_weak.upgrade().is_none());
    }
}

#[test]
fn contains_releases_unneeded_values_when_overwriting_the_only_aggregate() {
    for dictionary in [false, true] {
        for needle in [7, 8] {
            let mut worker = worker(Opcode::Contains, 1, 0, 1);
            let cell = worker.domain.new_cell(Value::Unit).expect("aggregate cell");
            let weak = cell.downgrade();
            let parent = if dictionary {
                Value::dict(vec![(Value::Integer(7), Value::Cell(cell))])
            } else {
                array(vec![Value::Integer(7), Value::Cell(cell)])
            };
            worker
                .store_register(0, Value::Integer(needle))
                .expect("needle");
            worker.store_register(1, parent).expect("only aggregate");
            assert_eq!(worker.domain.registered_count(), 1);
            worker.dispatch_one().expect("membership before overwrite");
            assert_eq!(worker.registers[1], Value::Boolean(needle == 7));
            assert_eq!(worker.domain.registered_count(), 0);
            assert!(weak.upgrade().is_none());
        }
    }
}

#[test]
fn aggregate_reads_release_old_destination_cells_only_on_success() {
    for opcode in [Opcode::IndexGet, Opcode::Contains] {
        for succeeds in [false, true] {
            let mut worker = worker(opcode, 2, 0, 1);
            let inputs = match (opcode, succeeds) {
                (Opcode::IndexGet, true) => [array(vec![Value::Integer(42)]), Value::Integer(0)],
                (Opcode::IndexGet, false) => [array(Vec::new()), Value::Integer(0)],
                (Opcode::Contains, true) => [Value::Integer(7), array(vec![Value::Integer(7)])],
                (Opcode::Contains, false) => [Value::Integer(7), Value::Str("text".into())],
                _ => unreachable!("aggregate read fixture"),
            };
            let expected = checked(opcode, &inputs[0], &inputs[1]);
            for (index, input) in inputs.into_iter().enumerate() {
                worker.store_register(index, input).expect("input");
            }
            let cell = worker
                .domain
                .new_cell(Value::Unit)
                .expect("old destination");
            let weak = cell.downgrade();
            worker
                .store_register(2, Value::Cell(cell))
                .expect("destination");
            let actual = worker.dispatch_one();
            match expected {
                Ok(expected) => {
                    actual.expect("successful read");
                    assert_eq!(worker.registers[2], expected);
                    assert_eq!(worker.domain.registered_count(), 0);
                    assert!(weak.upgrade().is_none());
                }
                Err(expected) => {
                    assert_eq!(
                        actual.err().expect("failed read"),
                        worker.runtime_error(expected.code, expected.message, expected.hint)
                    );
                    assert!(matches!(worker.registers[2], Value::Cell(_)));
                    assert_eq!(worker.domain.registered_count(), 1);
                    assert!(weak.upgrade().is_some());
                }
            }
            assert!(worker.register_is_initialized(2));
        }
    }
}
