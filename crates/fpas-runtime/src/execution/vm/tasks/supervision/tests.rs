//! Admission bounds and cancellation before executing any worker instruction.

use super::*;
use crate::execution::vm::{
    TaskResultPoll,
    tasks::{TaskScheduler, pool},
    worker::Worker,
};
use crate::{FunctionId, Intrinsic, SharedFunction, TaskIntrinsic, Value};
use std::sync::Arc;

pub(super) fn worker_and_group() -> (Worker, Value, Value) {
    let (program, errors) = fpas_parser::parse(
        "program T; uses Std.Tasks as Tasks; procedure Work(Token: Tasks.CancellationToken); begin panic('body executed'); end procedure; begin end.",
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
        image
            .function_target(FunctionId::try_from_index(index).expect("function"))
            .expect("verified function"),
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

#[test]
fn retries_keep_one_cell_domain_distinct_from_the_starting_task() {
    let (mut worker, group, callable) = worker_and_group();
    let result = worker
        .group_intrinsic(
            Intrinsic::Task(TaskIntrinsic::StartSupervisedTask),
            &[group, callable, Value::Integer(2), Value::Integer(0)],
            None,
        )
        .expect("start")
        .flatten()
        .expect("task");
    let Value::Task(id) = result else {
        panic!("task handle");
    };
    let scheduler = Arc::clone(worker.scheduler_ref().expect("scheduler"));
    let first = scheduler.try_dequeue().expect("initial attempt");
    let cell = first
        .domain
        .new_cell(Value::Integer(42))
        .expect("task-local cell");
    assert!(cell.read(&worker.domain).is_err());
    let _observation = first
        .domain
        .root_values(crate::RootKind::Host, vec![Value::Cell(cell.clone())]);
    let mut pending = Some(first);
    for _ in 0..3 {
        let task = pending
            .take()
            .unwrap_or_else(|| scheduler.try_dequeue().expect("retry"));
        assert_eq!(task.id, id);
        assert_eq!(
            cell.read(&task.domain).expect("same logical task"),
            Value::Integer(42)
        );
        pool::run_helped(&worker, task, Arc::clone(&scheduler)).expect("contained attempt failure");
    }
    assert!(scheduler.try_dequeue().is_none());
    assert!(matches!(
        scheduler.poll_result(id),
        TaskResultPoll::Failed(_)
    ));
}
