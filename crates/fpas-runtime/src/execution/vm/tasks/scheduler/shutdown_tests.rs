//! Shutdown wakeups cannot overtake a worker entering its condition-variable wait.

use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use super::TaskScheduler;

#[test]
fn shutdown_wakeup_waits_for_the_ready_queue_mutex() {
    let scheduler = Arc::new(TaskScheduler::new());
    let queue = scheduler.queue.lock().unwrap();
    assert!(!scheduler.is_shutdown());
    let notifier_scheduler = Arc::clone(&scheduler);
    let (finished_tx, finished_rx) = mpsc::channel();
    let notifier = std::thread::spawn(move || {
        notifier_scheduler.finish_main();
        finished_tx.send(()).unwrap();
    });

    // Pause the waiter after checking its predicate but before releasing the queue mutex.
    let deadline = Instant::now() + Duration::from_secs(5);
    while !scheduler.is_shutdown() && Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(scheduler.is_shutdown(), "notifier did not publish shutdown");
    let overtook_wait = finished_rx.recv_timeout(Duration::from_millis(100)).is_ok();
    let (queue, timeout) = scheduler
        .available
        .wait_timeout(queue, Duration::from_secs(1))
        .unwrap();
    drop(queue);
    notifier.join().unwrap();

    assert!(!overtook_wait, "shutdown notification overtook the wait");
    assert!(!timeout.timed_out(), "shutdown wakeup was lost");
    assert!(scheduler.dequeue().is_none());
}

#[test]
fn shutdown_retires_queued_and_late_task_roots_while_scheduler_remains_alive() {
    use crate::execution::vm::worker::Worker;
    use crate::{Value, execution::vm::TaskResultPoll};

    let (program, errors) =
        fpas_parser::parse("program LateTimer; begin const N := 1; discard N; end.");
    assert!(errors.is_empty());
    let image = Arc::new(fpas_compiler::compile(&program).expect("fixture"));
    let scheduler = Arc::new(TaskScheduler::new());
    for id in [7, 8] {
        let mut worker = Worker::new(image.clone())
            .expect("worker")
            .with_scheduler(Some(scheduler.clone()));
        worker.task_id = id;
        worker.retain_result = true;
        let domain = worker.domain.clone();
        let access = domain.access();
        let cell = domain.new_cell(Value::Unit).expect("cell");
        let weak = cell.downgrade();
        cell.replace(
            &domain,
            Value::Array(
                vec![
                    Value::Cell(cell.clone()),
                    Value::Str("x".repeat(65536).into()),
                ]
                .into(),
            ),
        )
        .expect("cycle");
        worker
            .store_register(0, Value::Cell(cell))
            .expect("live local");
        scheduler.register_result(id);
        scheduler.enqueue(worker.take_task_state());
        drop(access);
        if id == 7 {
            assert!(weak.upgrade().is_some(), "queued execution owns its cycle");
            scheduler.request_cancel();
        }
        assert!(
            !scheduler.has_queued(),
            "closed scheduler cannot retain late work"
        );
        assert!(
            weak.upgrade().is_none(),
            "queued or late work releases its cycle before scheduler drop"
        );
        assert_eq!(domain.collection_stats().live_cells, 0);
        assert!(matches!(
            scheduler.poll_result(id),
            TaskResultPoll::Failed(_)
        ));
    }
}
