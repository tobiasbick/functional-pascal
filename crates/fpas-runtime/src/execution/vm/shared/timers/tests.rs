//! Timer-driver shutdown notification synchronization.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;

use super::TaskTimers;

#[test]
fn shutdown_wakeup_waits_for_the_timer_queue_mutex() {
    let timers = Arc::new(TaskTimers::<u8>::new());
    let shutdown = Arc::new(AtomicBool::new(false));
    let sleeping = timers.sleeping.lock().unwrap();
    assert!(!shutdown.load(Ordering::Acquire));
    let notifier_timers = Arc::clone(&timers);
    let notifier_shutdown = Arc::clone(&shutdown);
    let (published_tx, published_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = mpsc::channel();
    let notifier = std::thread::spawn(move || {
        notifier_shutdown.store(true, Ordering::Release);
        published_tx.send(()).unwrap();
        notifier_timers.notify_shutdown();
        finished_tx.send(()).unwrap();
    });

    published_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let overtook_wait = finished_rx.recv_timeout(Duration::from_millis(100)).is_ok();
    let (sleeping, timeout) = timers
        .changed
        .wait_timeout(sleeping, Duration::from_secs(1))
        .unwrap();
    drop(sleeping);
    notifier.join().unwrap();

    assert!(
        !overtook_wait,
        "timer shutdown notification overtook the wait"
    );
    assert!(!timeout.timed_out(), "timer shutdown wakeup was lost");
    assert!(!timers.dispatch_next_due(&shutdown, |_| panic!("shutdown dispatched a timer")));
}
