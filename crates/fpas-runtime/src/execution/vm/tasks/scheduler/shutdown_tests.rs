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
    while !scheduler
        .draining
        .load(std::sync::atomic::Ordering::Acquire)
        && Instant::now() < deadline
    {
        std::thread::yield_now();
    }
    assert!(
        scheduler
            .draining
            .load(std::sync::atomic::Ordering::Acquire),
        "notifier did not request the drain"
    );
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

#[test]
fn last_timer_retirement_completes_a_pending_drain() {
    use crate::execution::vm::worker::Worker;
    use std::sync::atomic::Ordering;

    let (program, errors) = fpas_parser::parse("program TimerDrain; begin null; end.");
    assert!(errors.is_empty());
    let image = Arc::new(fpas_compiler::compile(&program).expect("fixture"));
    let scheduler = Arc::new(TaskScheduler::new());
    let mut worker = Worker::new(image)
        .expect("worker")
        .with_scheduler(Some(scheduler.clone()));
    worker.task_id = 7;
    scheduler.register_result(7);
    scheduler.enqueue(worker.take_task_state());
    let task = scheduler.try_dequeue().expect("running task");

    scheduler.request_cancel();
    assert!(
        !scheduler.is_shutdown(),
        "the claimed task still owns its retirement"
    );
    scheduler.schedule(task, 0);
    drop(scheduler.worker_ticket(7));
    assert!(
        !scheduler.is_shutdown(),
        "the timer still owns saved execution"
    );

    let (finished, completion) = mpsc::channel();
    let timer_scheduler = Arc::clone(&scheduler);
    let timer = std::thread::spawn(move || {
        timer_scheduler.timer_loop();
        finished.send(()).expect("observer");
    });
    let completed = completion.recv_timeout(Duration::from_secs(5));
    if completed.is_err() {
        // Release the timer fixture even when the drain invariant regresses.
        scheduler.shutdown.store(true, Ordering::Release);
        scheduler.timers.notify_shutdown();
    }
    timer.join().expect("timer thread");
    assert!(
        completed.is_ok(),
        "last timer retirement did not finish shutdown"
    );
    assert!(scheduler.is_shutdown());
}

#[test]
fn drain_keeps_a_saved_post_cleanup_failure_until_publication() {
    use crate::execution::vm::worker::Worker;

    let (program, errors) = fpas_parser::parse("program FailureDrain; begin null; end.");
    assert!(errors.is_empty());
    let image = Arc::new(fpas_compiler::compile(&program).expect("fixture"));
    let scheduler = Arc::new(TaskScheduler::new());
    let template = Worker::new(image)
        .expect("worker")
        .with_scheduler(Some(scheduler.clone()));
    let mut worker = template.pool_template();
    worker.task_id = 7;
    worker.retain_result = true;
    worker.cleanup.pending_failure = Some(crate::execution::vm::runtime_error(
        fpas_diagnostics::codes::RUNTIME_RESOURCE_RELEASE_FAILURE,
        "preserved cleanup failure",
        "fixture failure",
        crate::SourceLocation::new(1, 1),
    ));
    scheduler.register_result(7);
    scheduler.enqueue(worker.take_task_state());
    scheduler.request_cancel();
    assert!(
        !scheduler.is_shutdown(),
        "pending failure is still an owned outcome"
    );
    let task = scheduler
        .try_dequeue()
        .expect("retained failure continuation");
    let error = crate::execution::vm::tasks::pool::run_helped(&template, task, scheduler.clone())
        .expect_err("original failure must be published");
    assert_eq!(error.message, "preserved cleanup failure");
    assert_eq!(
        scheduler.first_error().expect("preserved failure").message,
        error.message
    );
    assert!(scheduler.is_shutdown());
}

#[test]
fn concurrent_stop_requests_cannot_finish_before_retained_work_is_requeued() {
    use crate::execution::vm::{tasks::pool, worker::Worker};
    use std::sync::atomic::Ordering;

    let (program, errors) = fpas_parser::parse("program T; begin null; end.");
    assert!(errors.is_empty());
    let image = Arc::new(fpas_compiler::compile(&program).expect("fixture"));
    let scheduler = Arc::new(TaskScheduler::new());
    let mut worker = Worker::new(image)
        .expect("worker")
        .with_scheduler(Some(scheduler.clone()));
    worker.task_id = 7;
    worker.retain_result = true;
    worker.enter_cleanup_scope();
    scheduler.register_result(7);
    let template = worker.pool_template();
    scheduler.enqueue(worker.take_task_state());

    let queue = scheduler.queue.lock().unwrap();
    let first_scheduler = scheduler.clone();
    let first = std::thread::spawn(move || first_scheduler.request_cancel());
    let deadline = Instant::now() + Duration::from_secs(5);
    while !scheduler.draining.load(Ordering::Acquire) && Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(scheduler.draining.load(Ordering::Acquire));
    let second_scheduler = scheduler.clone();
    let (sent, received) = mpsc::channel();
    let second = std::thread::spawn(move || {
        second_scheduler.request_cancel();
        sent.send(()).unwrap();
    });
    let overtook = received.recv_timeout(Duration::from_millis(100)).is_ok();
    drop(queue);
    first.join().unwrap();
    second.join().unwrap();
    assert!(
        !overtook,
        "second stop must wait for the initial drain setup"
    );
    assert!(!scheduler.is_shutdown(), "retained cleanup is still queued");
    let task = scheduler.try_dequeue().expect("retained cleanup task");
    pool::run_helped(&template, task, scheduler.clone()).expect("cancellation is contained");
    assert!(scheduler.is_shutdown());
}
