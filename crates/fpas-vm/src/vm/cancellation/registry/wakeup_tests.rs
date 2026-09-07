//! Cancellation subscriptions preserve notifications without owning the waiting operation.

use super::CancellationRegistry;
use crate::vm::shared::wakeups::WakeSignal;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn cancellation_between_probe_and_park_remains_latched() {
    let registry = CancellationRegistry::new();
    let source = registry.create_source();
    let signal = WakeSignal::new();
    let registration = registry.subscribe(source, &signal).expect("subscribe");
    assert!(!registry.is_cancelled(source).expect("probe"));
    registry.cancel(source).expect("cancel");
    assert!(signal.wait(Duration::ZERO));
    drop(registration);
    assert_eq!(
        registry
            .state(source)
            .expect("state")
            .changed
            .registration_count(),
        0
    );
}

#[test]
fn cancellation_before_subscription_is_visible_to_the_next_probe() {
    let registry = CancellationRegistry::new();
    let source = registry.create_source();
    registry.cancel(source).expect("cancel");
    let signal = WakeSignal::new();
    let _registration = registry.subscribe(source, &signal).expect("subscribe");
    assert!(registry.is_cancelled(source).expect("probe"));
}

#[test]
fn cancellation_racing_with_subscription_is_never_lost() {
    let registry = Arc::new(CancellationRegistry::new());
    for _ in 0..100 {
        let source = registry.create_source();
        let cancelling = Arc::clone(&registry);
        let notifier = std::thread::spawn(move || cancelling.cancel(source).expect("cancel"));
        let signal = WakeSignal::new();
        let registration = registry.subscribe(source, &signal).expect("subscribe");
        let observed =
            registry.is_cancelled(source).expect("probe") || signal.wait(Duration::from_secs(2));
        notifier.join().expect("join notifier");
        drop(registration);
        assert!(
            observed,
            "neither the probe nor the subscription observed cancellation"
        );
        assert_eq!(
            registry
                .state(source)
                .expect("state")
                .changed
                .registration_count(),
            0
        );
    }
}

#[test]
fn dropping_duplicate_registrations_releases_only_their_own_waits() {
    let registry = CancellationRegistry::new();
    let source = registry.create_source();
    let signal = WakeSignal::new();
    let weak_signal = Arc::downgrade(&signal);
    let first = registry
        .subscribe(source, &signal)
        .expect("first subscription");
    let second = registry
        .subscribe(source, &signal)
        .expect("second subscription");
    drop(first);
    registry.cancel(source).expect("cancel");
    assert!(signal.wait(Duration::ZERO));
    drop(second);
    drop(signal);
    assert!(weak_signal.upgrade().is_none());
    assert_eq!(
        registry
            .state(source)
            .expect("state")
            .changed
            .registration_count(),
        0
    );
}

#[test]
fn invalid_subscription_does_not_retain_its_signal() {
    let registry = CancellationRegistry::new();
    let signal = WakeSignal::new();
    assert!(registry.subscribe(1, &signal).is_err());
    assert_eq!(Arc::strong_count(&signal), 1);
}
