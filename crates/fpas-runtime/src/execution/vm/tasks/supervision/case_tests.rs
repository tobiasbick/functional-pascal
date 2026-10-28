//! Unused local selection callbacks end with their owner or abandoned attempt.

use std::sync::Arc;

use crate::{FunctionId, Intrinsic, SharedFunction, TaskIntrinsic, Value, WeakOwnedCell};

use crate::execution::vm::tasks::{TaskScheduler, pool};
use crate::execution::vm::worker::Worker;

fn worker() -> Worker {
    let (program, errors) = fpas_parser::parse(
        "program Cases; uses Std.Tasks as Tasks;
         procedure Work(Token: Tasks.CancellationToken); begin panic('attempt'); end procedure;
         begin var Counter := 0;
           const Callback: function(): integer := function(): integer
           begin Counter := Counter + 1; return Counter; end function;
         end.",
    );
    assert!(errors.is_empty(), "{errors:?}");
    Worker::new(Arc::new(
        fpas_compiler::compile(&program).expect("case fixture"),
    ))
    .expect("worker")
    .with_scheduler(Some(Arc::new(TaskScheduler::new())))
}

fn timer_case(worker: &mut Worker) -> (Value, WeakOwnedCell) {
    let index = worker
        .executable
        .executable()
        .functions
        .iter()
        .position(|info| info.arity == 0 && info.capture_count == 1)
        .expect("callback");
    let cell = worker
        .domain
        .new_cell(Value::Integer(7))
        .expect("callback capture");
    let weak = cell.downgrade();
    let callback = Value::task_owned_function(
        worker
            .executable
            .function_target(FunctionId::try_from_index(index).expect("id"))
            .expect("target"),
        vec![Value::Cell(cell)],
        worker.domain.id().clone(),
    );
    let handle = worker
        .selection_intrinsic(
            Intrinsic::Task(TaskIntrinsic::TimerCase),
            &[Value::Integer(60_000), callback],
            None,
        )
        .expect("case")
        .flatten()
        .expect("handle");
    (handle, weak)
}

fn assert_closed(worker: &mut Worker, case: Value, weak: &WeakOwnedCell) {
    assert!(
        weak.upgrade().is_none(),
        "unused callback capture must be released"
    );
    let result = worker
        .selection_intrinsic(
            Intrinsic::Task(TaskIntrinsic::CloseWaitCase),
            std::slice::from_ref(&case),
            None,
        )
        .expect("idempotent close")
        .flatten();
    assert_eq!(result, Some(Value::Boolean(false)));
    assert!(
        worker
            .selection_intrinsic(
                Intrinsic::Task(TaskIntrinsic::Select),
                &[Value::Array(vec![case].into())],
                None,
            )
            .is_err(),
        "stale case must not select"
    );
}

#[test]
fn terminal_publication_releases_cases_on_success_failure_and_cancellation() {
    for terminal in 0..3 {
        let mut worker = worker();
        worker.task_id = 7;
        let (case, weak) = timer_case(&mut worker);
        let scheduler = Arc::clone(worker.scheduler_ref().expect("scheduler"));
        scheduler.register_result(7);
        match terminal {
            0 => scheduler.store_result(7, Value::Unit),
            1 => {
                scheduler.store_failure(7, worker.group_error("failed"));
            }
            _ => scheduler.cancel_result(7),
        }
        assert_closed(&mut worker, case, &weak);
    }
}

#[test]
fn shutdown_releases_root_cases_and_closes_admission() {
    let mut worker = worker();
    let (case, weak) = timer_case(&mut worker);
    worker.scheduler_ref().expect("scheduler").finish_main();
    assert_closed(&mut worker, case, &weak);
    assert_eq!(worker.hosted.cases.close_all(), 0);
}

fn retry_fixture() -> (Worker, Arc<TaskScheduler>, Worker) {
    let mut parent = worker();
    let index = parent
        .executable
        .executable()
        .functions
        .iter()
        .position(|info| parent.executable.executable().strings.get(info.name) == Some("work"))
        .expect("work");
    let work = Value::Function(SharedFunction::unbound(
        parent
            .executable
            .function_target(FunctionId::try_from_index(index).expect("id"))
            .expect("target"),
        vec![],
    ));
    let group = parent
        .group_intrinsic(Intrinsic::Task(TaskIntrinsic::CreateTaskGroup), &[], None)
        .expect("create")
        .flatten()
        .expect("group");
    parent
        .group_intrinsic(
            Intrinsic::Task(TaskIntrinsic::StartSupervisedTask),
            &[group, work, Value::Integer(1), Value::Integer(0)],
            None,
        )
        .expect("start");
    let scheduler = Arc::clone(parent.scheduler_ref().expect("scheduler"));
    let child = parent.worker_for_task(scheduler.try_dequeue().expect("initial attempt"));
    (parent, scheduler, child)
}

#[test]
fn retry_releases_abandoned_cases_before_the_next_attempt_is_admitted() {
    let (parent, scheduler, mut child) = retry_fixture();
    let (case, weak) = timer_case(&mut child);
    let task = child.take_task_state();
    pool::run_helped(&parent, task, Arc::clone(&scheduler)).expect("retryable failure");
    assert_closed(&mut child, case, &weak);
    let retry = scheduler.try_dequeue().expect("retry queued after cleanup");
    assert_eq!(retry.domain.id(), child.domain.id());
    pool::run_helped(&parent, retry, Arc::clone(&scheduler)).expect("final contained failure");
    assert!(scheduler.try_dequeue().is_none());
}

#[test]
fn an_abandoned_attempt_cycle_survives_only_its_independent_inspection_root() {
    use crate::{MaintenanceOutcome, RootKind};
    let (parent, scheduler, mut child) = retry_fixture();
    let domain = child.domain.clone();
    let (case, weak) = timer_case(&mut child);
    let cell = weak.upgrade().unwrap();
    cell.replace(&domain, Value::option_some(Value::Cell(cell.clone())))
        .unwrap();
    let inspection = domain.root_values(RootKind::Inspection, vec![Value::Cell(cell)]);
    let task = child.take_task_state();
    pool::run_helped(&parent, task, scheduler.clone()).unwrap();
    assert!(matches!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert!(weak.upgrade().is_some());
    drop(inspection);
    assert!(matches!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert_closed(&mut child, case, &weak);
    let retry = scheduler.try_dequeue().unwrap();
    assert_eq!(retry.domain.id(), domain.id());
    pool::run_helped(&parent, retry, scheduler.clone()).unwrap();
    assert_eq!(domain.collection_stats().live_cells, 0);
}
