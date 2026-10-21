//! Direct execution roots remain complete across saved-state handoffs and re-entry.
use crate::vm::worker::Worker;
use fpas_bytecode::{TraceRoots, Value};
use std::sync::Arc;

fn worker() -> Worker {
    let (program, errors) =
        fpas_parser::parse("program Roots; begin const N := 1; discard N; end.");
    assert!(errors.is_empty());
    Worker::new(Arc::new(fpas_compiler::compile(&program).expect("image"))).expect("worker")
}
fn roots(holder: &impl TraceRoots) -> Vec<Value> {
    let mut values = Vec::new();
    holder
        .trace_roots(&mut |value| values.push(value.clone()))
        .expect("complete roots");
    values
}
#[test]
fn initialized_registers_completion_and_saved_state_share_the_enumerator() {
    let mut worker = worker();
    worker.store_register(0, Value::Integer(7)).expect("store");
    worker.pending_completion = Some(Ok(Value::Integer(8)));
    assert_eq!(roots(&worker), vec![Value::Integer(7), Value::Integer(8)]);
    let saved = worker.take_task_state();
    assert_eq!(roots(&saved), vec![Value::Integer(7), Value::Integer(8)]);
    assert!(roots(&worker).is_empty());
}
#[test]
fn native_caller_registration_survives_register_replacement_and_releases_cleanly() {
    let mut worker = worker();
    let cell = worker.domain.new_cell(Value::Integer(9)).expect("cell");
    let weak = cell.downgrade();
    worker.store_register(0, Value::Cell(cell)).expect("store");
    let native = worker.native_caller_roots().expect("native roots");
    worker.store_register(0, Value::Unit).expect("replace");
    assert!(weak.upgrade().is_some());
    let before = worker.domain.maintenance_generation();
    drop(native);
    assert!(weak.upgrade().is_none());
    assert!(worker.domain.maintenance_generation() > before);
}
#[test]
fn suspension_roots_cover_pending_sends_and_globals_report_contention() {
    let suspension = crate::vm::tasks::TaskSuspension::ChannelSend {
        handle: 1,
        token: None,
        value: Value::Integer(10),
        destination: None,
    };
    assert_eq!(roots(&suspension), vec![Value::Integer(10)]);
    let globals = Arc::new(std::sync::RwLock::new(vec![Some(Value::Integer(11))]));
    let source = super::roots::GlobalRoots(Arc::clone(&globals));
    let lock = globals.write().expect("writer");
    assert_eq!(
        source.trace_roots(&mut |_| {}),
        Err(fpas_bytecode::OwnershipError::Busy)
    );
    drop(lock);
    assert_eq!(roots(&source), vec![Value::Integer(11)]);
}
