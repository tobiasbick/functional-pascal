//! All public send modes reject local graphs before queueing or suspension.

use super::*;
use crate::vm::channels::{ReceiveState, SendState};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;
use std::sync::Arc;

fn worker() -> Worker {
    let (program, errors) = fpas_parser::parse("program SendOwnership; begin end.");
    assert!(errors.is_empty());
    Worker::new(Arc::new(fpas_compiler::compile(&program).expect("image"))).expect("worker")
}

#[test]
fn send_variants_reject_direct_and_wrapped_local_values_before_side_effects() {
    for debug_tasks in [false, true] {
        for operation in [
            TaskIntrinsic::Send,
            TaskIntrinsic::TrySend,
            TaskIntrinsic::SendWithTimeout,
            TaskIntrinsic::SendWithCancellation,
        ] {
            let mut worker = worker();
            worker.debug_tasks = debug_tasks;
            let channel = worker.hosted.channels.create(2).expect("channel");
            let cell = worker.domain.new_cell(Value::Integer(1)).expect("cell");
            for value in [
                Value::Cell(cell.clone()),
                Value::result_ok(Value::Array(vec![Value::Cell(cell.clone())].into())),
            ] {
                assert!(matches!(
                    worker
                        .hosted
                        .channels
                        .send(channel, Value::Integer(7), false, None),
                    Ok(SendState::Sent)
                ));
                let mut arguments = vec![Value::OpaqueHandle(channel), value];
                match operation {
                    TaskIntrinsic::SendWithTimeout => arguments.push(Value::Integer(0)),
                    TaskIntrinsic::SendWithCancellation => {
                        let source = worker.hosted.cancellations.create_source();
                        arguments.push(Value::OpaqueHandle(
                            worker.hosted.cancellations.token(source).expect("token"),
                        ));
                    }
                    _ => {}
                }
                let error = worker
                    .channel_intrinsic(Intrinsic::Task(operation), &arguments, None)
                    .expect_err("local send");
                assert_eq!(error.code, RUNTIME_INVALID_TASK);
                assert!(worker.task_suspension.is_none());
                assert!(!worker.suspend_requested);
                assert!(matches!(
                    worker.hosted.channels.receive(channel, false, None),
                    Ok(ReceiveState::Received(Value::Integer(7)))
                ));
                assert!(matches!(
                    worker.hosted.channels.receive(channel, false, None),
                    Ok(ReceiveState::Pending)
                ));
            }
        }
    }
}
