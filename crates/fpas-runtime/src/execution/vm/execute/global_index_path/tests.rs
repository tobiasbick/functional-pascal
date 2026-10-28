//! Ownership failures must precede consuming the global snapshot or its update window.

use std::sync::Arc;

use crate::{Instruction, MemoryDomain, Opcode, ReferenceRoot, Value, VariableReference};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

use crate::execution::vm::worker::Worker;

fn worker(local_globals: bool, root: Value, key: Value, replacement: Value) -> Worker {
    let (program, errors) = fpas_parser::parse(
        "program GlobalPath; var Items: array of integer := [1]; begin Items[0] := 2; end.",
    );
    assert!(errors.is_empty(), "{errors:?}");
    let image = Arc::new(fpas_compiler::compile(&program).expect("global fixture"));
    let mut worker = if local_globals {
        Worker::new_with_local_globals(image)
    } else {
        Worker::new(image)
    }
    .expect("worker");
    worker.global_slots_mut()[0] = Some(root.clone());
    // Deliberately bypass normal register admission to exercise this storage boundary itself.
    worker.registers = vec![root, key, replacement];
    worker.register_initialized = vec![true; 3];
    worker
}

fn foreign_graph() -> Value {
    let foreign = MemoryDomain::new();
    Value::option_some(Value::Array(
        vec![Value::Cell(
            foreign.new_cell(Value::Integer(7)).expect("foreign cell"),
        )]
        .into(),
    ))
}

fn reference(worker: &Worker) -> Value {
    Value::option_some(Value::Reference(Arc::new(VariableReference {
        root: ReferenceRoot::Cell(
            worker
                .domain
                .new_cell(Value::Integer(7))
                .expect("local cell"),
        ),
        path: Vec::new(),
    })))
}

fn assert_rejected_without_mutation(worker: &mut Worker) {
    let before_global = worker.global_slots()[0].clone();
    let before_registers = worker.registers.clone();
    let before_initialized = worker.register_initialized.clone();
    let instruction =
        Instruction::abc(Opcode::StoreGlobalIndexPath, 0, 0, 1, 1).expect("global index opcode");
    let error = worker
        .store_global_index_path(instruction.abc_payload())
        .expect_err("ownership must be checked before taking or modifying storage");
    assert_eq!(error.code, RUNTIME_INVALID_TASK);
    assert_eq!(worker.global_slots()[0], before_global, "global preserved");
    assert_eq!(worker.registers.len(), before_registers.len());
    for (actual, expected) in worker.registers.iter().zip(&before_registers) {
        assert_same_retained_value(actual, expected);
    }
    assert_eq!(worker.register_initialized, before_initialized);
}

// References deliberately have no value equality; retained identity proves they were not replaced.
fn assert_same_retained_value(actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::OptionSome(actual), Value::OptionSome(expected)) => {
            assert_same_retained_value(actual, expected);
        }
        (Value::Reference(actual), Value::Reference(expected)) => {
            assert!(
                Arc::ptr_eq(actual, expected),
                "reference identity preserved"
            );
        }
        _ => assert_eq!(actual, expected, "snapshot and update window preserved"),
    }
}

#[test]
fn store_global_index_path_rejects_nested_foreign_replacements_atomically() {
    for local_globals in [false, true] {
        let mut worker = worker(
            local_globals,
            Value::Array(vec![Value::Integer(1)].into()),
            Value::Integer(0),
            foreign_graph(),
        );
        assert_rejected_without_mutation(&mut worker);
    }
}

#[test]
fn store_global_index_path_rejects_escaping_references_atomically() {
    for local_globals in [false, true] {
        let mut worker = worker(
            local_globals,
            Value::Array(vec![Value::Integer(1)].into()),
            Value::Integer(0),
            Value::Unit,
        );
        worker.registers[2] = reference(&worker);
        assert_rejected_without_mutation(&mut worker);
    }
}

#[test]
fn store_global_index_path_rejects_foreign_missing_dictionary_keys_atomically() {
    for local_globals in [false, true] {
        let mut worker = worker(
            local_globals,
            Value::dict(Vec::new()),
            foreign_graph(),
            Value::Integer(9),
        );
        assert_rejected_without_mutation(&mut worker);
    }
}

#[test]
fn store_global_index_path_rejects_reference_dictionary_keys_atomically() {
    for local_globals in [false, true] {
        let mut worker = worker(
            local_globals,
            Value::dict(Vec::new()),
            Value::Unit,
            Value::Integer(9),
        );
        worker.registers[1] = reference(&worker);
        assert_rejected_without_mutation(&mut worker);
    }
}

#[test]
fn store_global_index_path_rejects_foreign_snapshot_roots_atomically() {
    for local_globals in [false, true] {
        let mut worker = worker(
            local_globals,
            Value::Array(vec![Value::Integer(1)].into()),
            Value::Integer(0),
            Value::Integer(9),
        );
        worker.registers[0] = Value::Array(vec![Value::Integer(1), foreign_graph()].into());
        assert_rejected_without_mutation(&mut worker);
    }
}
