//! Shared HTTP state must never retain a task-local graph.

use std::sync::Arc;

use fpas_bytecode::{MemoryDomain, RecordTypeId, RuntimeRecordLayout, SharedRecord};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

use super::*;

fn state_record(value: Value) -> Value {
    Value::Record(SharedRecord::new(
        Arc::new(RuntimeRecordLayout {
            record: RecordTypeId::new(0),
            type_name: "State".into(),
            fields: vec!["value".into()],
        }),
        vec![value],
    ))
}

#[test]
fn rejected_http_states_preserve_slots_and_existing_contents() {
    let domain = MemoryDomain::new();
    let cell = domain.new_cell(Value::Integer(7)).expect("cell");
    let (program, errors) = fpas_parser::parse("program HttpState; begin end.");
    assert!(errors.is_empty());
    let executable = fpas_compiler::compile(&program).expect("image");
    let target = executable
        .function_target(executable.executable().entry)
        .expect("target");
    for (reserve, store, load) in [
        (
            HttpIntrinsic::ReserveBodyStreamState,
            HttpIntrinsic::StoreBodyStreamState,
            HttpIntrinsic::LoadBodyStreamState,
        ),
        (
            HttpIntrinsic::ReserveSseDecoderState,
            HttpIntrinsic::StoreSseDecoderState,
            HttpIntrinsic::LoadSseDecoderState,
        ),
    ] {
        let registry = HttpStateRegistry::new();
        let original = state_record(Value::Integer(11));
        let slot = registry
            .reserve(reserve, original.clone())
            .expect("transfer")
            .expect("slot");
        assert_eq!(slot, 1);
        for local in [
            Value::Cell(cell.clone()),
            Value::option_some(Value::Array(vec![Value::Cell(cell.clone())].into())),
            Value::task_owned_function(target.clone(), Vec::new(), domain.id().clone()),
        ] {
            let replacement = state_record(local);
            assert!(registry.reserve(reserve, replacement.clone()).is_err());
            assert!(registry.store(store, slot, replacement).is_err());
            assert_eq!(registry.load(load, slot), Some(original.clone()));
        }
        assert_eq!(
            registry.reserve(reserve, original).expect("transfer"),
            Some(2)
        );
    }
}

#[test]
fn http_intrinsic_reports_local_state_before_reserving_or_overwriting() {
    let (program, errors) = fpas_parser::parse("program HttpState; begin end.");
    assert!(errors.is_empty());
    let worker =
        Worker::new(Arc::new(fpas_compiler::compile(&program).expect("image"))).expect("worker");
    let local = state_record(Value::Cell(
        worker.domain.new_cell(Value::Integer(3)).expect("cell"),
    ));
    for reserve in [
        HttpIntrinsic::ReserveBodyStreamState,
        HttpIntrinsic::ReserveSseDecoderState,
    ] {
        let error = worker
            .execute_http_state_intrinsic(
                Intrinsic::Http(reserve),
                std::slice::from_ref(&local),
                SourceLocation::new(1, 1),
            )
            .expect_err("local state must not enter a shared registry");
        assert_eq!(error.code, RUNTIME_INVALID_TASK);
    }
}
