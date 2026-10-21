//! Suspended native operation state remains a real collector root after migration.
use super::*;
use crate::vm::worker::Worker;
use fpas_bytecode::MaintenanceOutcome;
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
                values: vec![],
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
