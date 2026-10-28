//! Group admission, defensive retry inputs and final task values obey transfer policy.

use std::sync::Arc;

use crate::{Intrinsic, SharedFunction, TaskIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

use super::tests::worker_and_group;
use crate::execution::vm::TaskResultPoll;
use crate::execution::vm::tasks::pool;

#[test]
fn group_and_supervised_starts_reject_hidden_local_captures_without_membership() {
    for supervised in [false, true] {
        for wrapped in [false, true] {
            let (mut worker, group, callable) = worker_and_group();
            let Value::Function(function) = callable else {
                panic!("callable");
            };
            let cell = Value::Cell(worker.domain.new_cell(Value::Unit).expect("cell"));
            let capture = if wrapped {
                Value::option_some(Value::Array(vec![cell].into()))
            } else {
                cell
            };
            let work = Value::Function(SharedFunction::unbound(
                function.target().clone(),
                vec![capture],
            ));
            let operation = if supervised {
                TaskIntrinsic::StartSupervisedTask
            } else {
                TaskIntrinsic::StartTaskInGroup
            };
            let mut args = vec![group.clone(), work];
            if supervised {
                args.extend([Value::Integer(1), Value::Integer(0)]);
            }
            let error = worker
                .group_intrinsic(Intrinsic::Task(operation), &args, None)
                .expect_err("local work");
            assert_eq!(error.code, RUNTIME_INVALID_TASK);
            let scheduler = worker.scheduler_ref().expect("scheduler");
            assert!(scheduler.try_dequeue().is_none());
            assert!(matches!(scheduler.poll_result(1), TaskResultPoll::Unknown));
            assert_eq!(scheduler.alloc_id(), 1, "no ID allocated on rejection");
            let Value::OpaqueHandle(group) = group else {
                panic!("group");
            };
            scheduler
                .groups
                .begin_close(group, worker.task_id)
                .expect("close group");
            assert!(
                scheduler
                    .poll_group_close(group)
                    .expect("close")
                    .expect("no pending child")
                    .is_empty()
            );
        }
    }
}

#[test]
fn tampered_retry_inputs_fail_before_the_attempt_executes() {
    let (mut worker, group, callable) = worker_and_group();
    let task = worker
        .group_intrinsic(
            Intrinsic::Task(TaskIntrinsic::StartSupervisedTask),
            &[group, callable, Value::Integer(2), Value::Integer(0)],
            None,
        )
        .expect("start")
        .flatten()
        .expect("task");
    let Value::Task(id) = task else {
        panic!("task");
    };
    let scheduler = Arc::clone(worker.scheduler_ref().expect("scheduler"));
    let mut task = scheduler.try_dequeue().expect("initial admission");
    let cell = task.domain.new_cell(Value::Unit).expect("local cell");
    let supervision = task.supervision.as_mut().expect("retry inputs");
    supervision.function = SharedFunction::unbound(
        supervision.function.target().clone(),
        vec![Value::option_some(Value::Cell(cell))],
    );
    pool::run_helped(&worker, task, Arc::clone(&scheduler)).expect("owned transfer failure");
    assert!(
        matches!(scheduler.poll_result(id), TaskResultPoll::Failed(error) if error.code == RUNTIME_INVALID_TASK)
    );
    assert!(
        scheduler.try_dequeue().is_none(),
        "invalid work must not retry"
    );
}

#[test]
fn final_child_result_rejects_local_graph_before_any_success_publication() {
    for wrapped in [false, true] {
        let (mut worker, group, callable) = worker_and_group();
        let task = worker
            .group_intrinsic(
                Intrinsic::Task(TaskIntrinsic::StartTaskInGroup),
                &[group, callable],
                None,
            )
            .expect("start")
            .flatten()
            .expect("task");
        let Value::Task(id) = task else {
            panic!("task");
        };
        let scheduler = Arc::clone(worker.scheduler_ref().expect("scheduler"));
        let mut task = scheduler.try_dequeue().expect("child");
        let local = Value::Cell(task.domain.new_cell(Value::Integer(7)).expect("local cell"));
        let returned = if wrapped {
            Value::result_ok(Value::Array(vec![local].into()))
        } else {
            local
        };
        // Completed cleanup resumes through the same final publication path as a return.
        task.pending_completion = Some(Ok(returned));
        pool::run_helped(&worker, task, Arc::clone(&scheduler)).expect("contained invalid result");
        assert!(
            matches!(scheduler.poll_result(id), TaskResultPoll::Failed(error) if error.code == RUNTIME_INVALID_TASK)
        );
        assert!(!scheduler.is_shutdown());
    }
}
