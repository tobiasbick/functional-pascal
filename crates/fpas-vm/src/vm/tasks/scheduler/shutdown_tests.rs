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
