//! Constructor validation must not retain callbacks or admit invalid byte payloads.

use super::*;
use crate::{FunctionId, SharedFunction};
use std::sync::Arc;

fn worker() -> (Worker, Value) {
    let (program, errors) = fpas_parser::parse(
        "program ProcessCases;
         uses Std.Os.Proc as Proc;
         procedure Observe(Outcome: result of (integer, Proc.ProcError));
         begin null; end procedure;
         begin end.",
    );
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).unwrap();
    let id = image
        .executable()
        .functions
        .iter()
        .position(|info| info.arity == 1)
        .unwrap();
    let callback = Value::Function(SharedFunction::unbound(
        image.function_target(FunctionId::new(id as u16)).unwrap(),
        vec![],
    ));
    (Worker::new(Arc::new(image)).unwrap(), callback)
}

#[test]
fn invalid_bounds_and_bytes_never_register_a_case() {
    let (worker, callback) = worker();
    for operation in [ProcIntrinsic::ReadStdoutCase, ProcIntrinsic::ReadStderrCase] {
        for maximum in [
            Value::Integer(-1),
            Value::Integer(0),
            Value::Integer(1_048_577),
            Value::Boolean(true),
        ] {
            let error = worker
                .execute_process_case(
                    operation,
                    &[Value::OpaqueHandle(0), maximum, callback.clone()],
                )
                .unwrap_err();
            assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_INVALID_TASK);
        }
    }
    for bytes in [
        Value::Integer(0),
        Value::Array(vec![Value::Integer(-1)].into()),
        Value::Array(vec![Value::Integer(256)].into()),
        Value::Array(vec![Value::Boolean(true)].into()),
        Value::Array(vec![Value::Integer(0); 1_048_577].into()),
    ] {
        let error = worker
            .execute_process_case(
                ProcIntrinsic::WriteStdinCase,
                &[Value::OpaqueHandle(0), bytes, callback.clone()],
            )
            .unwrap_err();
        assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_INVALID_TASK);
    }
    assert_eq!(worker.hosted.cases.close_owner(worker.task_id), 0);
}

#[test]
fn closed_process_is_a_callback_outcome_instead_of_a_constructor_failure() {
    let (worker, callback) = worker();
    assert!(matches!(
        worker
            .execute_process_case(
                ProcIntrinsic::WaitStatusCase,
                &[Value::OpaqueHandle(0), callback,]
            )
            .unwrap(),
        Some(Value::OpaqueHandle(_))
    ));
    assert!(matches!(
        worker
            .poll_process_case(0, &ProcessCase::WaitStatus)
            .unwrap(),
        Some(Value::ResultError(_))
    ));
    assert_eq!(worker.hosted.cases.close_owner(worker.task_id), 1);
}
