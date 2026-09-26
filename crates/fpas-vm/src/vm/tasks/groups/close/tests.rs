//! Timeout must preserve ownership, results, cancellation state, and the original deadline.

use super::*;
use crate::vm::{
    TaskResultPoll,
    tasks::{TaskClock, TaskScheduler},
};
use fpas_bytecode::{Intrinsic, TaskIntrinsic};

fn worker_and_group() -> (Worker, u64) {
    let (program, errors) = fpas_parser::parse(
        "program T; uses Std.Task; begin var G: TaskGroup := CreateTaskGroup(); CloseTaskGroupWithTimeout(G, 0) end.",
    );
    assert!(errors.is_empty(), "{errors:?}");
    let mut worker = Worker::new(Arc::new(fpas_compiler::compile(&program).expect("compile")))
        .expect("worker")
        .with_scheduler(Some(Arc::new(TaskScheduler::new())));
    let group = worker
        .group_intrinsic(Intrinsic::Task(TaskIntrinsic::CreateTaskGroup), &[], None)
        .expect("create")
        .flatten()
        .expect("group");
    let Value::OpaqueHandle(id) = group else {
        panic!("group handle")
    };
    (worker, id)
}

#[test]
fn invalid_timeout_and_wrong_owner_do_not_seal_or_cancel_the_group() {
    let (mut worker, id) = worker_and_group();
    let scheduler = Arc::clone(worker.scheduler_ref().expect("scheduler"));
    let token = scheduler.groups.token(id).expect("token");
    for invalid in [Value::Integer(-1), Value::Boolean(false)] {
        assert!(
            worker
                .group_intrinsic(
                    Intrinsic::Task(TaskIntrinsic::CloseTaskGroupWithTimeout),
                    &[Value::OpaqueHandle(id), invalid],
                    None
                )
                .is_err()
        );
        assert!(
            !worker
                .hosted
                .cancellations
                .is_cancelled(token)
                .expect("active token")
        );
    }
    worker.task_id = 42;
    assert!(
        worker
            .start_group_close(id, Some(Duration::ZERO), None)
            .is_err()
    );
    assert!(
        !worker
            .hosted
            .cancellations
            .is_cancelled(token)
            .expect("active token")
    );
    scheduler
        .groups
        .enroll(id, 0, 1)
        .expect("admission still open");
}

#[test]
fn repeated_timeouts_retain_children_results_reports_and_token_until_join() {
    let (mut worker, id) = worker_and_group();
    let scheduler = Arc::clone(worker.scheduler_ref().expect("scheduler"));
    let token = scheduler.groups.token(id).expect("token");
    for child in 1..=2 {
        scheduler.groups.enroll(id, 0, child).expect("child");
        scheduler.register_result(child);
    }
    scheduler.store_result(
        1,
        Value::result_error(Value::Str("retained failure".into())),
    );
    for _ in 0..100 {
        assert_eq!(
            worker
                .start_group_close(id, Some(Duration::ZERO), None)
                .expect("close"),
            Some(Value::result_error(Value::Str(
                "Task group close timed out".into()
            )))
        );
        assert_eq!(scheduler.groups.token(id).expect("retained token"), token);
        assert!(
            worker
                .hosted
                .cancellations
                .is_cancelled(token)
                .expect("cancelled")
        );
        assert!(matches!(scheduler.poll_result(2), TaskResultPoll::Pending));
    }
    assert!(
        scheduler.groups.enroll(id, 0, 3).is_err(),
        "admission reopened"
    );
    assert!(
        matches!(
            scheduler.poll_result(1),
            TaskResultPoll::Available(Value::ResultError(_))
        ),
        "timeout consumed the completed child's result"
    );
    scheduler.store_result(2, Value::Integer(42));
    let Some(Value::ResultOk(report)) = worker
        .start_group_close(id, Some(Duration::ZERO), None)
        .expect("join")
    else {
        panic!("completed group must win over zero timeout")
    };
    let Value::Array(reports) = report.as_ref() else {
        panic!("failure array")
    };
    assert_eq!(reports.len(), 1, "consuming a result lost the group report");
    assert!(scheduler.groups.token(id).is_err());
    assert!(worker.hosted.cancellations.is_cancelled(token).is_err());
    assert!(matches!(scheduler.poll_result(2), TaskResultPoll::Consumed));
    assert_eq!(
        worker
            .start_group_close(id, Some(Duration::ZERO), None)
            .expect("repeated close"),
        Some(Value::result_ok(Value::Array(vec![].into())))
    );
}

#[test]
fn pending_close_preserves_its_deadline_across_debugger_probes() {
    let (mut worker, id) = worker_and_group();
    let scheduler = Arc::clone(worker.scheduler_ref().expect("scheduler"));
    scheduler.groups.enroll(id, 0, 1).expect("child");
    scheduler.register_result(1);
    worker.debug_tasks = true;
    worker.task_clock = Some(Arc::new(TaskClock::manual()));
    assert!(
        worker
            .start_group_close(id, Some(Duration::from_millis(5)), None)
            .expect("start")
            .is_none()
    );
    for _ in 0..4 {
        worker.task_clock_ref().wait(Duration::from_millis(1));
        assert!(!worker.poll_task_suspension().expect("pending"));
        assert!(matches!(
            worker.task_suspension,
            Some(TaskSuspension::GroupClose {
                deadline_millis: Some(5),
                ..
            })
        ));
    }
    worker.task_clock_ref().wait(Duration::from_millis(1));
    assert!(worker.poll_task_suspension().expect("timeout"));
    assert!(worker.task_suspension.is_none());
    assert!(
        scheduler.groups.token(id).is_ok(),
        "timeout released ownership"
    );
}

#[test]
fn shutdown_is_not_reported_as_timeout_or_successful_join() {
    let (mut worker, id) = worker_and_group();
    let scheduler = Arc::clone(worker.scheduler_ref().expect("scheduler"));
    scheduler.groups.enroll(id, 0, 1).expect("child");
    scheduler.register_result(1);
    scheduler.finish_main();
    let error = worker
        .start_group_close(id, Some(Duration::ZERO), None)
        .expect_err("shutdown");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN);
    assert!(scheduler.groups.token(id).is_ok());
}

#[test]
fn timed_root_close_never_executes_queued_child_code_inline() {
    use fpas_bytecode::{FunctionId, SharedFunction};
    let (mut worker, id) = worker_and_group();
    let scheduler = Arc::clone(worker.scheduler_ref().expect("scheduler"));
    scheduler.groups.enroll(id, 0, 1).expect("child");
    scheduler.register_result(1);
    let info = &worker.executable.executable().functions[0];
    let function = SharedFunction::unbound(FunctionId::new(0), "must remain queued", vec![]);
    let task = crate::vm::tasks::TaskState::entry(1, &function, info, [], true);
    scheduler.enqueue(task);
    let started = std::time::Instant::now();
    assert_eq!(
        worker
            .start_group_close(id, Some(Duration::from_millis(2)), None)
            .expect("timeout"),
        Some(Value::result_error(Value::Str(
            "Task group close timed out".into()
        )))
    );
    assert!(started.elapsed() >= Duration::from_millis(2));
    assert_eq!(scheduler.try_dequeue().expect("child still queued").id, 1);
}

#[test]
fn completion_racing_timeout_is_either_joined_or_retained_for_retry() {
    let (mut worker, _) = worker_and_group();
    let scheduler = Arc::clone(worker.scheduler_ref().expect("scheduler"));
    for _ in 0..200 {
        let group = scheduler
            .groups
            .create(0, || worker.hosted.cancellations.create_owned())
            .expect("group");
        let child = scheduler.alloc_id();
        scheduler.groups.enroll(group, 0, child).expect("child");
        scheduler.register_result(child);
        std::thread::scope(|scope| {
            let publisher = scope.spawn(|| scheduler.store_result(child, Value::Integer(42)));
            let result = worker
                .start_group_close(group, Some(Duration::ZERO), None)
                .expect("close");
            if matches!(result, Some(Value::ResultError(_))) {
                assert!(scheduler.groups.token(group).is_ok());
            } else {
                assert!(matches!(result, Some(Value::ResultOk(_))));
                assert!(scheduler.groups.token(group).is_err());
            }
            publisher.join().expect("publication");
        });
        assert_eq!(
            worker
                .start_group_close(group, Some(Duration::ZERO), None)
                .expect("retry"),
            Some(Value::result_ok(Value::Array(vec![].into())))
        );
        assert!(matches!(
            scheduler.poll_result(child),
            TaskResultPoll::Consumed
        ));
    }
}
