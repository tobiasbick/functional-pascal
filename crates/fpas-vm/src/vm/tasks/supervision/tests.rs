//! Admission bounds and cancellation before executing any worker instruction.

use super::*;
use crate::vm::{
    TaskResultPoll,
    tasks::{TaskScheduler, pool},
    worker::Worker,
};
use fpas_bytecode::{FunctionId, Intrinsic, SharedFunction, TaskIntrinsic, Value};
use std::sync::Arc;

fn worker_and_group() -> (Worker, Value, Value) {
    let (program, errors) = fpas_parser::parse(
        "program T; uses Std.Task; procedure Work(Token: CancellationToken); begin panic('body executed') end; begin end.",
    );
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).expect("compile");
    let index = image
        .executable()
        .functions
        .iter()
        .position(|info| image.executable().strings.get(info.name) == Some("work"))
        .expect("work");
    let callable = Value::Function(SharedFunction::unbound(
        FunctionId::try_from_index(index).expect("function"),
        "work".into(),
        vec![],
    ));
    let mut worker = Worker::new(Arc::new(image))
        .expect("worker")
        .with_scheduler(Some(Arc::new(TaskScheduler::new())));
    let group = worker
        .group_intrinsic(Intrinsic::Task(TaskIntrinsic::CreateTaskGroup), &[], None)
        .expect("create")
        .flatten()
        .expect("group");
    (worker, group, callable)
}

#[test]
fn invalid_supervisor_bounds_do_not_publish_a_child() {
    let (mut worker, group, callable) = worker_and_group();
    for (retries, backoff) in [(-1, 0), (1024, 0), (0, -1), (0, 60001)] {
        let error = worker
            .group_intrinsic(
                Intrinsic::Task(TaskIntrinsic::StartSupervisedTask),
                &[
                    group.clone(),
                    callable.clone(),
                    Value::Integer(retries),
                    Value::Integer(backoff),
                ],
                None,
            )
            .expect_err("invalid policy");
        assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_INVALID_TASK);
        assert!(
            worker
                .scheduler_ref()
                .expect("scheduler")
                .try_dequeue()
                .is_none()
        );
    }
    assert!(RetryPolicy::new(0, 0).is_ok());
    assert!(RetryPolicy::new(1023, 60000).is_ok());
}

#[test]
fn cancellation_before_first_admission_prevents_worker_execution() {
    let (mut worker, group, callable) = worker_and_group();
    let result = worker
        .group_intrinsic(
            Intrinsic::Task(TaskIntrinsic::StartSupervisedTask),
            &[
                group.clone(),
                callable,
                Value::Integer(2),
                Value::Integer(60000),
            ],
            None,
        )
        .expect("start")
        .flatten()
        .expect("task");
    let Value::Task(id) = result else {
        panic!("task handle");
    };
    worker
        .group_intrinsic(
            Intrinsic::Task(TaskIntrinsic::CancelTaskGroup),
            &[group],
            None,
        )
        .expect("cancel");
    let scheduler = Arc::clone(worker.scheduler_ref().expect("scheduler"));
    let task = scheduler.try_dequeue().expect("pending child");
    pool::run_helped(&worker, task, Arc::clone(&scheduler)).expect("contained cancellation");
    assert!(
        matches!(scheduler.poll_result(id), TaskResultPoll::Failed(error) if error.code == fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED)
    );
    assert!(scheduler.try_dequeue().is_none());
}
