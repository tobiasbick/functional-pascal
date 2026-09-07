//! Group reports survive result consumption but never certify synthetic shutdown completion.

use super::{TaskScheduler, tests::failure};
use crate::vm::{TaskResultPoll, cancellation::CancellationRegistry};
use fpas_bytecode::Value;
use fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN;

#[test]
fn group_report_survives_wait_and_close_discards_every_child_result() {
    let scheduler = TaskScheduler::new();
    let cancellations = CancellationRegistry::new();
    let group = scheduler
        .groups
        .create(0, || cancellations.create_owned())
        .expect("group");
    for task in 1..=3 {
        scheduler.groups.enroll(group, 0, task).expect("child");
        scheduler.register_result(task);
    }
    scheduler.store_result(1, Value::result_error(Value::Str("ordinary".into())));
    assert!(matches!(
        scheduler.poll_result(1),
        TaskResultPoll::Available(Value::ResultError(_))
    ));
    assert!(scheduler.store_failure(2, failure("owned panic")));
    scheduler.store_result(3, Value::Integer(42));
    scheduler.groups.begin_close(group, 0).expect("close");
    let report = scheduler
        .poll_group_close(group)
        .expect("close")
        .expect("joined");
    assert_eq!(
        report
            .iter()
            .map(|failure| failure.task)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert!(scheduler.results.lock().expect("results").is_empty());
    for task in 1..=3 {
        assert!(matches!(
            scheduler.poll_result(task),
            TaskResultPoll::Consumed
        ));
    }
}

#[test]
fn synthetic_shutdown_results_cannot_certify_group_join() {
    let scheduler = TaskScheduler::new();
    let cancellations = CancellationRegistry::new();
    let group = scheduler
        .groups
        .create(0, || cancellations.create_owned())
        .expect("group");
    scheduler.groups.enroll(group, 0, 1).expect("child");
    scheduler.register_result(1);
    scheduler.groups.begin_close(group, 0).expect("close");
    assert!(
        scheduler
            .poll_group_close(group)
            .expect("pending close")
            .is_none()
    );
    scheduler.finish_main();
    let Err(error) = scheduler.poll_group_close(group) else {
        panic!("synthetic completion must not report a successful join");
    };
    assert_eq!(error.code, RUNTIME_VM_SHUTDOWN);
    assert!(scheduler.groups.token(group).is_ok());
}

#[test]
fn close_racing_failure_publication_keeps_failure_owned() {
    let scheduler = TaskScheduler::new();
    let cancellations = CancellationRegistry::new();
    for _ in 0..200 {
        let group = scheduler
            .groups
            .create(0, || cancellations.create_owned())
            .expect("group");
        let task = scheduler.alloc_id();
        scheduler.groups.enroll(group, 0, task).expect("child");
        scheduler.register_result(task);
        scheduler.groups.begin_close(group, 0).expect("close");
        std::thread::scope(|scope| {
            let publish = scope.spawn(|| scheduler.store_failure(task, failure("owned panic")));
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            loop {
                if let Some(report) = scheduler.poll_group_close(group).expect("close") {
                    assert_eq!(report.len(), 1);
                    assert_eq!(report[0].task, task);
                    break;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "child completion was not published"
                );
                std::thread::yield_now();
            }
            assert!(publish.join().expect("publisher"));
        });
    }
}
