//! Wrapper transfers preserve aliases and reject invalid sources before consuming them.
use super::support::*;
use crate::execution::vm::worker::Worker;
use crate::{Opcode, ReferenceRoot, Value, VariableReference};
use std::sync::Arc;

fn worker(opcode: Opcode, destination: u16, consume: u8) -> Worker {
    Worker::new(Arc::new(verified(
        vec![abc_aux(opcode, destination, 0, 0, consume), return_unit()],
        vec![],
        vec!["root", "test.fpas"],
        2,
    )))
    .expect("worker")
}

fn payload(value: &Value) -> &Value {
    match value {
        Value::OptionSome(value) | Value::ResultOk(value) | Value::ResultError(value) => value,
        _ => panic!("expected wrapper"),
    }
}

#[test]
fn consuming_wrapper_construction_moves_nested_storage_and_preserves_cell_aliases() {
    for opcode in [Opcode::MakeSome, Opcode::MakeOk, Opcode::MakeError] {
        for destination in [0, 1] {
            for consume in [0, 1] {
                let mut worker = worker(opcode, destination, consume);
                let cell = worker.domain.new_cell(Value::Integer(7)).unwrap();
                let input = Value::option_some(Value::Cell(cell.clone()));
                let address = std::ptr::from_ref(payload(&input));
                worker.store_register(0, input).unwrap();
                worker.dispatch_one().unwrap();
                assert_eq!(
                    worker.register_is_initialized(0),
                    consume == 0 || destination == 0
                );
                let nested = payload(&worker.registers[usize::from(destination)]);
                assert_eq!(std::ptr::eq(address, payload(nested)), consume == 1);
                let Value::Cell(alias) = payload(nested) else {
                    panic!("cell");
                };
                assert!(alias.ptr_eq(&cell));
                assert_eq!(cell.read(&worker.domain).unwrap(), Value::Integer(7));
            }
        }
    }
}

#[test]
fn failed_consuming_construction_preserves_foreign_and_ephemeral_sources() {
    for ephemeral in [false, true] {
        let mut worker = worker(Opcode::MakeSome, 1, 1);
        let foreign = crate::MemoryDomain::new();
        let value = if ephemeral {
            Value::Reference(Arc::new(VariableReference {
                root: ReferenceRoot::Global {
                    slot: 0,
                    owner: worker.domain.id().clone(),
                },
                path: vec![],
            }))
        } else {
            Value::Cell(foreign.new_cell(Value::Unit).unwrap())
        };
        worker.store_register(0, value.clone()).unwrap();
        worker.store_register(1, Value::Integer(99)).unwrap();
        let error = worker.dispatch_one().err().expect("invalid stored value");
        assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_INVALID_TASK);
        assert!(worker.register_is_initialized(0));
        match (&worker.registers[0], &value) {
            (Value::Reference(actual), Value::Reference(expected)) => {
                assert!(Arc::ptr_eq(actual, expected))
            }
            (Value::Cell(actual), Value::Cell(expected)) => assert!(actual.ptr_eq(expected)),
            _ => panic!("source changed after rejected construction"),
        }
        assert_eq!(worker.registers[1], Value::Integer(99));
    }
}
