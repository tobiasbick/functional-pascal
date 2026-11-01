//! Typed transport, partial failures and actual VM execution.
use super::*;
use crate::execution::vm::{
    Vm,
    debug::{DebugRunResult, DebugSession},
};
use serde_json::json;
use std::sync::Arc;

fn image(source: &str) -> Arc<crate::VerifiedExecutable> {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    Arc::new(fpas_compiler::compile(&program).unwrap())
}

#[test]
fn typed_transport_preserves_absence_backend_failures_and_integer_boundaries() {
    let worker = Worker::new(image(
        "program T; uses Std.Os.System as System; begin discard System.Snapshot(); end.",
    ))
    .unwrap();
    let location = SourceLocation::new(1, 1);
    assert!(matches!(
        worker.host_info_value(Shape::Integer, &json!(i64::MAX), "Count", location),
        Ok(Value::Integer(i64::MAX))
    ));
    assert!(
        matches!(worker.host_info_value(Shape::Integer, &json!(u64::MAX), "Count", location), Err(Failure::Range(field)) if field == "Count")
    );
    assert!(matches!(
        worker.host_info_value(
            Shape::Option(&Shape::Real),
            &Json::Null,
            "Temperature",
            location
        ),
        Ok(Value::OptionNone)
    ));
    let failed = worker
        .host_info_value(
            Shape::Result(&Shape::Integer),
            &json!({"Error": "unsupported test reading"}),
            "Cpu.Count",
            location,
        )
        .unwrap_or_else(|_| panic!("typed section"));
    let Value::ResultError(error) = failed else {
        panic!("failure preserved");
    };
    let Value::Enum(error) = error.as_ref() else {
        panic!("nominal error");
    };
    assert_eq!(error.body().layout.type_name, "Std.Os.System.SystemError");
    assert_eq!(error.body().layout.variant, "QueryFailed");
    assert_eq!(
        error.body().values,
        vec![
            Value::Str("Cpu.Count".into()),
            Value::Str("unsupported test reading".into())
        ]
    );
}

#[test]
fn host_queries_run_in_vm_and_debugger_but_seeded_scheduling_rejects_them() {
    let source = include_str!("../../../../../../../tests/stdlib/host_info_test.fpas");
    let image = image(source);
    Vm::new((*image).clone()).run().unwrap();
    let mut debug = DebugSession::with_limits_and_schedule(
        (*image).clone(),
        vec![],
        Default::default(),
        Default::default(),
        None,
    )
    .unwrap();
    assert!(matches!(
        debug.continue_execution().unwrap(),
        DebugRunResult::Terminated(_)
    ));
    let mut seeded = DebugSession::with_limits_and_schedule(
        (*image).clone(),
        vec![],
        Default::default(),
        Default::default(),
        Some(17),
    )
    .unwrap();
    let DebugRunResult::Stopped(stop) = seeded.continue_execution().unwrap() else {
        panic!("seeded host query must be rejected");
    };
    assert_eq!(
        stop.diagnostic.unwrap().code,
        fpas_diagnostics::codes::RUNTIME_SCHEDULE_UNSUPPORTED_EFFECT
    );
}
