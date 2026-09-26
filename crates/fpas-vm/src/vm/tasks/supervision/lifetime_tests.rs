//! Retry templates and parked attempts must not retain completed worker captures.

use std::sync::{Arc, Weak};

use fpas_bytecode::{
    FunctionId, Intrinsic, RecordTypeId, RuntimeRecordLayout, SharedFunction, SharedRecord,
    TaskIntrinsic, Value, VerifiedExecutable,
};

use crate::vm::tasks::{TaskScheduler, pool};
use crate::vm::worker::Worker;

fn image(outcome: &str) -> Arc<VerifiedExecutable> {
    let (program, errors) = fpas_parser::parse(&format!(
        "program Captures; uses Std.Task;
         type Payload = record Number: integer; end;
         begin
           var Captured: Payload := record Number := 42; end;
           var Group: TaskGroup := CreateTaskGroup();
           StartSupervisedTask(Group, function(Token: CancellationToken): result of integer, string
           begin
             if Captured.Number <> 42 then panic('capture changed');
             return {outcome}
           end, 2, 0)
         end."
    ));
    assert!(errors.is_empty(), "{errors:?}");
    Arc::new(fpas_compiler::compile(&program).expect("capture fixture"))
}

fn start(
    image: Arc<VerifiedExecutable>,
    retries: i64,
    backoff: i64,
) -> (Worker, Value, Arc<TaskScheduler>, Weak<RuntimeRecordLayout>) {
    let executable = image.executable();
    let function = executable
        .functions
        .iter()
        .position(|info| info.capture_count == 1)
        .expect("one-capture worker");
    let record = executable
        .records
        .iter()
        .position(|layout| {
            executable
                .strings
                .get(layout.name)
                .is_some_and(|name| name.eq_ignore_ascii_case("payload"))
        })
        .expect("payload layout");
    let layout = Arc::new(RuntimeRecordLayout {
        record: RecordTypeId::try_from_index(record).expect("record id"),
        type_name: "payload".into(),
        fields: vec!["number".into()],
    });
    let weak = Arc::downgrade(&layout);
    let work = Value::Function(SharedFunction::unbound(
        FunctionId::try_from_index(function).expect("function id"),
        "worker",
        vec![Value::Record(SharedRecord::new(
            layout,
            vec![Value::Integer(42)],
        ))],
    ));
    let scheduler = Arc::new(TaskScheduler::new());
    let mut worker = Worker::new(image)
        .expect("worker")
        .with_scheduler(Some(Arc::clone(&scheduler)));
    let group = worker
        .group_intrinsic(Intrinsic::Task(TaskIntrinsic::CreateTaskGroup), &[], None)
        .expect("create")
        .flatten()
        .expect("group");
    worker
        .group_intrinsic(
            Intrinsic::Task(TaskIntrinsic::StartSupervisedTask),
            &[
                group.clone(),
                work,
                Value::Integer(retries),
                Value::Integer(backoff),
            ],
            None,
        )
        .expect("start");
    assert!(
        weak.upgrade().is_some(),
        "fixture lost capture before admission"
    );
    (worker, group, scheduler, weak)
}

#[test]
fn successful_supervision_releases_captures_before_group_close() {
    let image = image("Ok(42)");
    for _ in 0..100 {
        let (worker, _group, scheduler, weak) = start(Arc::clone(&image), 2, 60000);
        let task = scheduler.try_dequeue().expect("initial attempt");
        let id = task.id;
        pool::run_helped(&worker, task, Arc::clone(&scheduler)).expect("successful attempt");
        assert!(
            weak.upgrade().is_none(),
            "successful attempt retained capture"
        );
        assert!(
            matches!(scheduler.poll_result(id), crate::vm::TaskResultPoll::Available(value)
            if value == Value::result_ok(Value::Integer(42)))
        );
        assert!(
            scheduler.try_dequeue().is_none(),
            "successful worker retried"
        );
    }
}

#[test]
fn exhausted_supervision_releases_captures_before_group_close() {
    let image = image("Error('retry')");
    for _ in 0..100 {
        let (worker, _group, scheduler, weak) = start(Arc::clone(&image), 2, 0);
        for attempt in 0..3 {
            let task = scheduler.try_dequeue().expect("next attempt");
            pool::run_helped(&worker, task, Arc::clone(&scheduler)).expect("contained error");
            assert_eq!(weak.upgrade().is_some(), attempt < 2, "attempt {attempt}");
        }
        assert!(scheduler.try_dequeue().is_none(), "retry limit exceeded");
    }
}

#[test]
fn cancelled_supervision_releases_captures_without_executing_work() {
    let image = image("Error('retry')");
    for _ in 0..100 {
        let (mut worker, group, scheduler, weak) = start(Arc::clone(&image), 2, 60000);
        worker
            .group_intrinsic(
                Intrinsic::Task(TaskIntrinsic::CancelTaskGroup),
                &[group],
                None,
            )
            .expect("cancel");
        let task = scheduler.try_dequeue().expect("pending child");
        pool::run_helped(&worker, task, Arc::clone(&scheduler)).expect("contained cancellation");
        assert!(
            weak.upgrade().is_none(),
            "cancelled attempt retained capture"
        );
        assert!(scheduler.try_dequeue().is_none());
    }
}

#[test]
fn shutdown_releases_supervision_captures_parked_in_backoff() {
    let image = image("Error('retry')");
    for _ in 0..100 {
        let (worker, _group, scheduler, weak) = start(Arc::clone(&image), 2, 60000);
        let task = scheduler.try_dequeue().expect("initial attempt");
        pool::run_helped(&worker, task, Arc::clone(&scheduler)).expect("retry parked");
        assert!(weak.upgrade().is_some(), "backoff must own retry inputs");
        assert!(scheduler.try_dequeue().is_none(), "backoff bypassed timer");
        scheduler.finish_main();
        assert!(weak.upgrade().is_none(), "cancelled timer retained capture");
    }
}

#[test]
fn dropping_queued_supervision_releases_captures() {
    let image = image("Error('retry')");
    for _ in 0..100 {
        let (worker, _group, scheduler, weak) = start(Arc::clone(&image), 2, 0);
        drop(worker);
        assert!(weak.upgrade().is_some(), "queued child owns its capture");
        drop(scheduler);
        assert!(
            weak.upgrade().is_none(),
            "scheduler teardown retained capture"
        );
    }
}
