//! Suspended native operation state remains a real collector root after migration.
use super::*;
use crate::MaintenanceOutcome;
use crate::execution::vm::worker::Worker;
use std::sync::Arc;

#[test]
fn saved_continuation_accumulator_retains_a_local_cycle_until_discarded() {
    let (program, errors) = fpas_parser::parse("program Reduction; begin end.");
    assert!(errors.is_empty());
    let mut worker = Worker::new(Arc::new(fpas_compiler::compile(&program).unwrap())).unwrap();
    let domain = worker.domain.clone();
    let cell = domain.new_cell(Value::Unit).unwrap();
    let weak = cell.downgrade();
    let target = worker.executable.function_target(worker.function).unwrap();
    let accumulator = Value::function(target.clone(), vec![Value::Cell(cell.clone())]);
    cell.replace(&domain, accumulator.clone()).unwrap();
    drop(cell);
    let Value::Function(callback) = Value::function(target, vec![]) else {
        unreachable!();
    };
    worker
        .callback_continuations
        .push(CallbackContinuation::new(
            callback,
            None,
            CallbackOperation::ArrayReduce {
                values: vec![].into(),
                next: 0,
                accumulator,
            },
            (worker.function, worker.current_address),
        ));
    let mut saved = worker.take_task_state();
    drop(worker);
    assert!(matches!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert!(weak.upgrade().is_some());
    {
        let _access = domain.access();
        saved.callback_continuations.clear();
        saved.publish_execution();
    }
    assert!(matches!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert!(weak.upgrade().is_none());
}

#[test]
fn saved_dictionary_snapshot_retains_keys_and_values_after_original_changes() {
    let (program, errors) = fpas_parser::parse("program Reduction; begin end.");
    assert!(errors.is_empty());
    let mut worker = Worker::new(Arc::new(fpas_compiler::compile(&program).unwrap())).unwrap();
    let domain = worker.domain.clone();
    let key = domain.new_cell(Value::Unit).unwrap();
    let value = domain.new_cell(Value::Unit).unwrap();
    key.replace(&domain, Value::Cell(key.clone())).unwrap();
    value.replace(&domain, Value::Cell(value.clone())).unwrap();
    let weak_key = key.downgrade();
    let weak_value = value.downgrade();
    let mut original: crate::SharedDict = vec![(Value::Cell(key), Value::Cell(value))].into();
    let snapshot = original.clone();
    original.clear();
    drop(original);
    let target = worker.executable.function_target(worker.function).unwrap();
    let Value::Function(callback) = Value::function(target, vec![]) else {
        unreachable!()
    };
    worker
        .callback_continuations
        .push(CallbackContinuation::new(
            callback,
            None,
            CallbackOperation::DictReduce(DictionaryReduction::new(snapshot, Value::Unit)),
            (worker.function, worker.current_address),
        ));
    let mut saved = worker.take_task_state();
    drop(worker);
    assert!(matches!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert!(weak_key.upgrade().is_some());
    assert!(weak_value.upgrade().is_some());
    {
        let _access = domain.access();
        saved.callback_continuations.clear();
        saved.publish_execution();
    }
    assert!(matches!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert!(weak_key.upgrade().is_none());
    assert!(weak_value.upgrade().is_none());
}
