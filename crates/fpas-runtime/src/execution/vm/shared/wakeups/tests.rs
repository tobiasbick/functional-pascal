//! Registration lifetime and notification race regressions.

use super::*;

#[test]
fn notification_before_parking_is_latched() {
    let source = Arc::new(WakeSource::default());
    let signal = WakeSignal::new();
    let _registration = source.subscribe(&signal);
    source.notify();
    assert!(signal.wait(Duration::ZERO));
}

#[test]
fn either_source_can_wake_one_signal_without_retaining_losers() {
    let first = Arc::new(WakeSource::default());
    let second = Arc::new(WakeSource::default());
    for source in [&first, &second] {
        let signal = WakeSignal::new();
        let registrations = [first.subscribe(&signal), second.subscribe(&signal)];
        source.notify();
        assert!(signal.wait(Duration::ZERO));
        drop(registrations);
        assert!(first.subscriptions.lock().unwrap().is_empty());
        assert!(second.subscriptions.lock().unwrap().is_empty());
    }
}

#[test]
fn dropping_one_duplicate_registration_keeps_the_other_active() {
    let source = Arc::new(WakeSource::default());
    let signal = WakeSignal::new();
    let first = source.subscribe(&signal);
    let second = source.subscribe(&signal);
    drop(first);
    assert_eq!(source.subscriptions.lock().unwrap().len(), 1);
    source.notify();
    assert!(signal.wait(Duration::ZERO));
    drop(second);
    assert!(source.subscriptions.lock().unwrap().is_empty());
}

#[test]
fn timeout_and_unwinding_remove_registrations() {
    let source = Arc::new(WakeSource::default());
    drop(source.subscribe(&WakeSignal::new()));
    let initial_capacity = source.subscriptions.lock().unwrap().capacity();
    for _ in 0..1000 {
        let signal = WakeSignal::new();
        let registration = source.subscribe(&signal);
        assert!(!signal.wait(Duration::ZERO));
        drop(registration);
    }
    let result = std::panic::catch_unwind(|| {
        let _registration = source.subscribe(&WakeSignal::new());
        panic!("simulated failure during a wait")
    });
    assert!(result.is_err());
    assert!(source.subscriptions.lock().unwrap().is_empty());
    assert_eq!(
        source.subscriptions.lock().unwrap().capacity(),
        initial_capacity
    );
}

#[test]
fn source_does_not_retain_wait_signal_after_registration_drops() {
    let source = Arc::new(WakeSource::default());
    let signal = WakeSignal::new();
    let weak = Arc::downgrade(&signal);
    let registration = source.subscribe(&signal);
    drop(signal);
    assert!(weak.upgrade().is_some());
    drop(registration);
    assert!(weak.upgrade().is_none());
}

#[test]
fn notification_racing_registration_cleanup_remains_safe() {
    let source = Arc::new(WakeSource::default());
    std::thread::scope(|scope| {
        scope.spawn(|| {
            for _ in 0..2000 {
                source.notify();
            }
        });
        for _ in 0..2000 {
            let signal = WakeSignal::new();
            let registration = source.subscribe(&signal);
            source.notify();
            assert!(signal.wait(Duration::ZERO));
            drop(registration);
        }
    });
    assert!(source.subscriptions.lock().unwrap().is_empty());
}
