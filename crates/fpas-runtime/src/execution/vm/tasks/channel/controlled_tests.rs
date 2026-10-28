//! Real suspended operations observe readiness before expiry without timing races.

use super::*;
use crate::execution::vm::tasks::errors::TaskErrorKind;
use crate::execution::vm::tasks::{TaskClock, TaskScheduler};
use std::sync::Arc;
use std::time::Duration;

fn worker() -> Worker {
    let (program, errors) = fpas_parser::parse(
        "program T; uses Std.Tasks as Tasks; begin const Q: channel of integer := Tasks.CreateChannel(1); discard Tasks.TryReceive(Q); end.",
    );
    assert!(errors.is_empty());
    let mut worker = Worker::new(Arc::new(fpas_compiler::compile(&program).unwrap())).unwrap();
    worker.debug_tasks = true;
    worker.task_clock = Some(Arc::new(TaskClock::manual()));
    worker.scheduler = Some(Arc::new(TaskScheduler::new()));
    worker
}

#[test]
fn resumed_receive_commits_ready_value_even_after_its_original_deadline() {
    let mut worker = worker();
    let channel = worker.hosted.channels.create(1).unwrap();
    assert!(matches!(
        worker
            .channel_intrinsic(
                Intrinsic::Task(TaskIntrinsic::ReceiveWithTimeout),
                &[Value::OpaqueHandle(channel), Value::Integer(5)],
                None
            )
            .unwrap(),
        Some(None)
    ));
    worker.task_clock_ref().wait(Duration::from_millis(10));
    worker
        .hosted
        .channels
        .send(channel, Value::Integer(42), false, None)
        .unwrap();
    assert!(worker.poll_task_suspension().unwrap());
    assert!(matches!(
        worker
            .hosted
            .channels
            .receive(channel, false, None)
            .unwrap(),
        crate::execution::vm::channels::ReceiveState::Pending
    ));
}

#[test]
fn resumed_send_commits_ready_capacity_even_after_its_original_deadline() {
    let mut worker = worker();
    let channel = worker.hosted.channels.create(1).unwrap();
    worker
        .hosted
        .channels
        .send(channel, Value::Integer(1), false, None)
        .unwrap();
    assert!(matches!(
        worker
            .channel_intrinsic(
                Intrinsic::Task(TaskIntrinsic::SendWithTimeout),
                &[
                    Value::OpaqueHandle(channel),
                    Value::Integer(42),
                    Value::Integer(5)
                ],
                None
            )
            .unwrap(),
        Some(None)
    ));
    worker.task_clock_ref().wait(Duration::from_millis(10));
    worker
        .hosted
        .channels
        .receive(channel, false, None)
        .unwrap();
    assert!(worker.poll_task_suspension().unwrap());
    assert!(matches!(
        worker
            .hosted
            .channels
            .receive(channel, false, None)
            .unwrap(),
        crate::execution::vm::channels::ReceiveState::Received(Value::Integer(42))
    ));
}

#[test]
fn cancelled_operations_never_transfer_ready_data_and_validate_channel_identity() {
    let mut worker = worker();
    let source = worker.hosted.cancellations.create_source();
    let token = worker.hosted.cancellations.token(source).unwrap();
    worker.hosted.cancellations.cancel(source).unwrap();
    let channel = worker.hosted.channels.create(1).unwrap();
    worker
        .hosted
        .channels
        .send(channel, Value::Integer(42), false, None)
        .unwrap();
    let received = worker
        .channel_intrinsic(
            Intrinsic::Task(TaskIntrinsic::ReceiveWithCancellation),
            &[Value::OpaqueHandle(channel), Value::OpaqueHandle(token)],
            None,
        )
        .unwrap();
    assert_eq!(
        received,
        Some(Some(
            worker
                .task_outcome_error(
                    TaskErrorKind::Cancelled,
                    "Std.Tasks.ReceiveWithCancellation"
                )
                .unwrap()
        ))
    );
    assert!(matches!(
        worker
            .hosted
            .channels
            .receive(channel, false, None)
            .unwrap(),
        crate::execution::vm::channels::ReceiveState::Received(Value::Integer(42))
    ));
    let sent = worker
        .channel_intrinsic(
            Intrinsic::Task(TaskIntrinsic::SendWithCancellation),
            &[
                Value::OpaqueHandle(channel),
                Value::Integer(7),
                Value::OpaqueHandle(token),
            ],
            None,
        )
        .unwrap();
    assert_eq!(
        sent,
        Some(Some(
            worker
                .task_outcome_error(TaskErrorKind::Cancelled, "Std.Tasks.SendWithCancellation")
                .unwrap()
        ))
    );
    assert!(matches!(
        worker
            .hosted
            .channels
            .receive(channel, false, None)
            .unwrap(),
        crate::execution::vm::channels::ReceiveState::Pending
    ));
    assert!(
        worker
            .channel_intrinsic(
                Intrinsic::Task(TaskIntrinsic::ReceiveWithCancellation),
                &[Value::OpaqueHandle(99), Value::OpaqueHandle(token)],
                None
            )
            .is_err()
    );
}
