//! Relative deadlines preserve zero-timeout semantics and never round a positive budget down.

use super::*;

#[test]
fn realtime_deadlines_do_not_expire_before_the_requested_budget() {
    let clock = TaskClock::realtime();
    for _ in 0..16 {
        let started = Instant::now();
        let budget = Duration::from_millis(1);
        let deadline = clock.deadline_after(budget);
        while clock.now_millis() < deadline {
            std::thread::yield_now();
        }
        assert!(started.elapsed() >= budget, "deadline rounded down");
    }
    assert!(clock.deadline_after(Duration::ZERO) <= clock.now_millis());
}

#[test]
fn manual_clock_deadlines_preserve_exact_ticks_and_immediate_zero() {
    let clock = TaskClock::manual();
    clock.wait(Duration::from_millis(3));
    assert_eq!(clock.deadline_after(Duration::ZERO), 3);
    assert_eq!(clock.deadline_after(Duration::from_millis(2)), 5);
    assert_eq!(clock.deadline_after(Duration::from_nanos(1)), 4);
}

#[test]
fn task_clock_deadlines_saturate_instead_of_wrapping() {
    let clock = TaskClock {
        mode: TaskClockMode::Manual(AtomicU64::new(u64::MAX - 1)),
    };
    assert_eq!(clock.deadline_after(Duration::MAX), u64::MAX);
}
